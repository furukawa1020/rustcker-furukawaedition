//! WSL2 distribution management for Rustker Desktop.
//! Ensures a lightweight Alpine-based distro is available for container execution.

use anyhow::{Context, Result};
use tracing::{info, warn};

/// Manages WSL2 distributions for Rustker Desktop.
pub struct WslManager {
    /// Name of the managed distro (e.g. "rustker-alpine")
    pub distro_name: String,
    /// Directory where the distro rootfs is stored
    pub install_dir: std::path::PathBuf,
}

impl WslManager {
    /// Create a new WslManager with the given distro name and install path.
    pub fn new(distro_name: impl Into<String>, install_dir: std::path::PathBuf) -> Self {
        Self {
            distro_name: distro_name.into(),
            install_dir,
        }
    }

    /// List all installed WSL distributions.
    pub async fn list_distros(&self) -> Result<Vec<String>> {
        let output = tokio::process::Command::new("wsl.exe")
            .args(["--list", "--quiet"])
            .output()
            .await
            .context("Failed to run wsl.exe --list")?;

        // WSL outputs UTF-16LE on Windows; we decode and clean up null bytes
        let raw = output.stdout;
        let text = decode_wsl_output(&raw);
        
        let distros = text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(distros)
    }

    /// Ensure the managed distro exists, importing Alpine Linux if not present.
    pub async fn ensure_distro(&self) -> Result<()> {
        let distros = self.list_distros().await?;
        
        if distros.iter().any(|d| d.eq_ignore_ascii_case(&self.distro_name)) {
            info!("WSL distro '{}' is already installed.", self.distro_name);
            return Ok(());
        }

        info!("WSL distro '{}' not found. Setting up Alpine Linux...", self.distro_name);
        self.import_alpine().await
    }

    /// Download and import a minimal Alpine Linux rootfs as a WSL distro.
    async fn import_alpine(&self) -> Result<()> {
        // Alpine mini rootfs URL (latest stable amd64)
        let alpine_url = "https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.1-x86_64.tar.gz";
        let tarball_path = self.install_dir.join("alpine-rootfs.tar.gz");
        let distro_dir = self.install_dir.join(&self.distro_name);

        // Ensure install directory exists
        tokio::fs::create_dir_all(&distro_dir).await
            .context("Failed to create distro install directory")?;

        // Download the rootfs tarball using curl (available on modern Windows)
        info!("Downloading Alpine Linux rootfs from {}...", alpine_url);
        let status = tokio::process::Command::new("curl")
            .args(["-L", "-o", tarball_path.to_str().unwrap_or("alpine-rootfs.tar.gz"), alpine_url])
            .status()
            .await
            .context("Failed to run curl to download Alpine rootfs")?;

        if !status.success() {
            anyhow::bail!("curl failed to download Alpine rootfs");
        }

        // Import the tarball as a WSL distro
        info!("Importing '{}' into WSL2...", self.distro_name);
        let status = tokio::process::Command::new("wsl.exe")
            .args([
                "--import",
                &self.distro_name,
                distro_dir.to_str().unwrap_or("."),
                tarball_path.to_str().unwrap_or("alpine-rootfs.tar.gz"),
                "--version", "2",
            ])
            .status()
            .await
            .context("Failed to run wsl.exe --import")?;

        if !status.success() {
            anyhow::bail!("wsl.exe --import failed for distro '{}'", self.distro_name);
        }

        // Clean up the tarball
        if let Err(e) = tokio::fs::remove_file(&tarball_path).await {
            warn!("Failed to clean up tarball: {}", e);
        }

        info!("WSL distro '{}' has been successfully set up.", self.distro_name);
        Ok(())
    }

    /// Convert a Windows path to a WSL path using `wslpath`.
    pub async fn to_wsl_path(&self, windows_path: &str) -> Result<String> {
        let output = tokio::process::Command::new("wsl.exe")
            .args(["-d", &self.distro_name, "--", "wslpath", "-u", windows_path])
            .output()
            .await
            .context("Failed to run wslpath")?;

        let wsl_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if wsl_path.is_empty() {
            anyhow::bail!("wslpath returned empty path for '{}'", windows_path);
        }
        Ok(wsl_path)
    }
}

/// Decode WSL output which may be UTF-16LE with null bytes.
fn decode_wsl_output(raw: &[u8]) -> String {
    // Try to detect UTF-16LE (BOM: FF FE)
    if raw.len() >= 2 && raw[0] == 0xFF && raw[1] == 0xFE {
        let wide: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&wide);
    }
    // Fallback: strip null bytes (common in WSL output on some Windows versions)
    String::from_utf8_lossy(raw)
        .chars()
        .filter(|c| *c != '\0')
        .collect()
}
