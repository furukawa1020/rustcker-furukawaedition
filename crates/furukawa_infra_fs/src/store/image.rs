use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use bytes::Bytes;
use furukawa_domain::image::Digest;
use furukawa_common::diagnostic::Diagnosable;

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
}
