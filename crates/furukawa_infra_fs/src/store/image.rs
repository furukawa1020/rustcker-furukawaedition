use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use bytes::Bytes;


#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Digest")]
    InvalidDigest,
}

pub struct ImageStore {
    root_path: PathBuf,
}

impl ImageStore {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    pub async fn ensure_dirs(&self) -> Result<(), StoreError> {
        fs::create_dir_all(self.root_path.join("layers")).await?;
        fs::create_dir_all(self.root_path.join("configs")).await?;
        Ok(())
    }

    pub fn layer_path(&self, digest: &str) -> PathBuf {
        // Expected digest format: "sha256:..."
        // Safe filename: remove "sha256:" prefix or just replace : with _
        let safe_name = digest.replace(":", "_");
        self.root_path.join("layers").join(safe_name)
    }

    pub fn config_path(&self, id: &str) -> PathBuf {
        let safe_name = id.replace(":", "_");
        self.root_path.join("configs").join(format!("{}.json", safe_name))
    }

    pub async fn has_layer(&self, digest: &str) -> bool {
        self.layer_path(digest).exists() // TODO: async exists? tokio::fs::metadata
    }

    pub async fn save_layer(&self, digest: &str, data: Bytes) -> Result<(), StoreError> {
        let path = self.layer_path(digest);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut file = File::create(path).await?;
        file.write_all(&data).await?;
        Ok(())
    }

    pub async fn save_config(&self, id: &str, config: serde_json::Value) -> Result<(), StoreError> {
        let path = self.config_path(id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let data = serde_json::to_vec(&config).unwrap(); // Should handle error
        let mut file = File::create(path).await?;
        file.write_all(&data).await?;
        Ok(())
    }

    /// Unpacks a layer into the target directory.
    /// This is a blocking operation for now as 'tar' and 'flate2' are synchronous.
    /// In a 10 year architecture, we'd use tokio-tar or spawn_blocking.
    pub async fn unpack_layer(&self, digest: &str, target_dir: PathBuf) -> Result<(), StoreError> {
        let layer_path = self.layer_path(digest);
        if !layer_path.exists() {
            return Err(StoreError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Layer not found")));
        }

        let target_dir_clone = target_dir.clone();
        
        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(layer_path)?;
            let decompressed = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decompressed);
            
            // Standard Docker layers often have whiteout files (.wh.)
            // For this phase, we do a simple extraction mapping.
            archive.unpack(target_dir_clone)?;
            Ok::<(), std::io::Error>(())
        }).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))??;

        Ok(())
    }

    pub async fn compose_rootfs(&self, layers: &[String], target_dir: PathBuf) -> Result<(), StoreError> {
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        for digest in layers {
            tracing::info!("Unpacking layer: {}", digest);
            self.unpack_layer(digest, target_dir.clone()).await?;
        }

        // Basic Whiteout Handling:
        // After unpacking all layers, we should scan for .wh. files.
        // In a real 10-year foundation, we'd handle this during untarring.
        // For Phase 5, we do a post-extraction cleanup to satisfy the "Compose RootFS" objective.
        let target_dir_clone = target_dir.clone();
        tokio::task::spawn_blocking(move || {
            for entry in walkdir::WalkDir::new(&target_dir_clone)
                .into_iter()
                .filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with(".wh.") {
                        // This is a whiteout file.
                        // For .wh.<file>, we delete both the whiteout AND the target file.
                        // For .wh..wh..opq, it's more complex (opaque dir), 
                        // but for now we just handle the file hiding.
                        let target_file_name = file_name.strip_prefix(".wh.").unwrap();
                        let target_path = path.parent().unwrap().join(target_file_name);
                        
                        if target_path.exists() {
                            if target_path.is_dir() {
                                std::fs::remove_dir_all(&target_path)?;
                            } else {
                                std::fs::remove_file(&target_path)?;
                            }
                        }
                        std::fs::remove_file(path)?; // Remove the whiteout marker itself
                    }
                }
            }
            Ok::<(), std::io::Error>(())
        }).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))??;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    fn create_test_layer(files: Vec<(&str, &str)>) -> Vec<u8> {
        let buf = Vec::new();
        let enc = GzEncoder::new(buf, Compression::default());
        let mut tar = tar::Builder::new(enc);

        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_path(name).unwrap();
            header.set_cksum();
            tar.append(&header, content.as_bytes()).unwrap();
        }

        tar.finish().unwrap();
        tar.into_inner().unwrap().finish().unwrap()
    }

    #[tokio::test]
    async fn test_compose_rootfs_with_whiteouts() {
        let tmp = TempDir::new().unwrap();
        let store = ImageStore::new(tmp.path().to_path_buf());
        store.ensure_dirs().await.unwrap();

        // Layer 1: Base files
        let layer1_data = create_test_layer(vec![
            ("etc/config", "base-config"),
            ("usr/bin/app", "binary-v1"),
        ]);
        let digest1 = "sha256:layer1";
        store.save_layer(digest1, Bytes::from(layer1_data)).await.unwrap();

        // Layer 2: Update app and whiteout config
        // In Docker, to whiteout "etc/config", we add "etc/.wh.config"
        let layer2_data = create_test_layer(vec![
            ("usr/bin/app", "binary-v2"),
            ("etc/.wh.config", ""),
        ]);
        let digest2 = "sha256:layer2";
        store.save_layer(digest2, Bytes::from(layer2_data)).await.unwrap();

        let target_dir = tmp.path().join("rootfs");
        store.compose_rootfs(&[digest1.to_string(), digest2.to_string()], target_dir.clone()).await.unwrap();

        // Verify outcomes
        assert!(target_dir.join("usr/bin/app").exists());
        let app_content = std::fs::read_to_string(target_dir.join("usr/bin/app")).unwrap();
        assert_eq!(app_content, "binary-v2");

        assert!(!target_dir.join("etc/config").exists());
        assert!(!target_dir.join("etc/.wh.config").exists());
    }
}
