mod process_control;

use async_trait::async_trait;
use furukawa_domain::container::{Container, Created, Running};
use furukawa_domain::container::runtime::ContainerRuntime;
use furukawa_common::diagnostic::Error;
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

#[derive(Default, Clone)]
pub struct ProcessRuntime;

#[async_trait]
impl ContainerRuntime for ProcessRuntime {
    async fn start(&self, container: &Container<Created>) -> Result<Running, Error> {
        let config = container.config();
        
        info!("Starting container {} with command: {:?}", container.id(), config.cmd);
        
        let program = if config.cmd.is_empty() {
             "sh"
        } else {
             &config.cmd[0]
        };
        
        let args = if config.cmd.len() > 1 {
            &config.cmd[1..]
        } else {
            &[]
        };

        // Create logs directory
        let log_dir = std::path::Path::new("furukawa_logs");
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir).map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        }

        let log_path = log_dir.join(format!("{}.log", container.id()));
        let log_file = std::fs::File::create(&log_path)
            .map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;

        // Clone file handle for both stdout and stderr (merging them for now)
        let stdout_file = log_file.try_clone().map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        let stderr_file = log_file; // Move the original handle

        // Strict: Use Command::new
        let child = Command::new(program)
            .args(args)
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|e| Error::new(RuntimeError::SpawnFailed(e)))?;
            
        let pid = child.id().ok_or_else(|| Error::new(RuntimeError::NoPid))?;
        
        info!("Container started with PID: {}", pid);

        Ok(Running {
            pid,
            started_at: time::OffsetDateTime::now_utc(),
        })
    }

    async fn stop(&self, container: &Container<Running>) -> Result<(), Error> {
        let pid = container.state().pid;
        process_control::stop_container(pid)
    }
}

pub struct WslRuntime {
    pub image_store: std::sync::Arc<furukawa_infra_fs::store::image::ImageStore>,
    pub metadata_store: std::sync::Arc<dyn furukawa_domain::image::store::ImageMetadataStore>,
    pub containers_root: std::path::PathBuf,
    pub distro: String,
}

#[async_trait]
impl ContainerRuntime for WslRuntime {
    async fn start(&self, container: &Container<Created>) -> Result<Running, Error> {
        let config = container.config();
        
        info!("Starting WSL container {} with image: {}", container.id(), config.image);

        // 1. Resolve Image Metadata
        let metadata = self.metadata_store.get(&config.image).await
            .map_err(|e| Error::new(RuntimeError::ImageResolutionFailed(e.to_string())))?
            .ok_or_else(|| Error::new(RuntimeError::ImageResolutionFailed("Image not found".into())))?;

        // 2. Prepare RootFS
        let container_dir = self.containers_root.join(container.id());
        let rootfs_dir = container_dir.join("rootfs");
        
        if !rootfs_dir.exists() {
            info!("Composing rootfs for container {} at {:?}", container.id(), rootfs_dir);
            self.image_store.compose_rootfs(&metadata.layers, rootfs_dir.clone()).await
                .map_err(|e| Error::new(RuntimeError::RootfsCompositionFailed(e.to_string())))?;
        }

        // 3. Construct WSL command
        // Path mapping: C:\path\to\rootfs -> /mnt/c/path/to/rootfs
        // This is a naive mapping, real impl would use 'wslpath' or check mount points
        let rootfs_str = rootfs_dir.to_string_lossy().replace("\\", "/");
        let wsl_path = if let Some(stripped) = rootfs_str.strip_prefix("C:") {
            format!("/mnt/c{}", stripped)
        } else if let Some(stripped) = rootfs_str.strip_prefix("c:") {
            format!("/mnt/c{}", stripped)
        } else {
            // Fallback for other drives or relative paths
            rootfs_str.clone()
        };

        let program = if config.cmd.is_empty() { "sh" } else { &config.cmd[0] };
        let args = if config.cmd.len() > 1 { &config.cmd[1..] } else { &[] };

        // Construct command: wsl.exe -d <distro> -u root -- chroot <wsl_path> <program> <args>
        let mut wsl_cmd = Command::new("wsl.exe");
        wsl_cmd.arg("-d").arg(&self.distro)
               .arg("-u").arg("root")
               .arg("--")
               .arg("chroot").arg(&wsl_path)
               .arg(program);
        
        for arg in args {
            wsl_cmd.arg(arg);
        }

        // Setup logs
        let log_dir = std::path::Path::new("furukawa_logs");
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir).map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        }
        let log_path = log_dir.join(format!("{}.log", container.id()));
        let log_file = std::fs::File::create(&log_path).map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        let stdout_file = log_file.try_clone().map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        let stderr_file = log_file;

        info!("Spawning WSL process...");
        let child = wsl_cmd
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|e| Error::new(RuntimeError::SpawnFailed(e)))?;

        let pid = child.id().ok_or_else(|| Error::new(RuntimeError::NoPid))?;
        
        // 4. Port Forwarding (Netsh)
        if !config.port_mappings.is_empty() {
             // Get WSL IP
             let output = std::process::Command::new("wsl.exe")
                .arg("-d").arg(&self.distro)
                .arg("hostname").arg("-I")
                .output()
                .map_err(|e| Error::new(RuntimeError::PortForwardingFailed(format!("Failed to get WSL IP: {}", e))))?;
             
             let ip_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
             let wsl_ip = ip_str.split_whitespace().next().ok_or_else(|| Error::new(RuntimeError::PortForwardingFailed("No WSL IP found".into())))?;

             for mapping in &config.port_mappings {
                 info!("Setting up port forward: {} -> {}:{}", mapping.host_port, wsl_ip, mapping.container_port);
                 // netsh interface portproxy add v4tov4 listenport=8080 listenaddress=0.0.0.0 connectport=80 connectaddress=172.x.x.x
                 let status = std::process::Command::new("netsh")
                    .arg("interface").arg("portproxy")
                    .arg("add").arg("v4tov4")
                    .arg(format!("listenport={}", mapping.host_port))
                    .arg("listenaddress=0.0.0.0")
                    .arg(format!("connectport={}", mapping.container_port))
                    .arg(format!("connectaddress={}", wsl_ip))
                    .status()
                    .map_err(|e| Error::new(RuntimeError::PortForwardingFailed(format!("Netsh failed: {}", e))))?;

                 if !status.success() {
                     tracing::warn!("Failed to add port proxy for port {}. Check if running as Admin.", mapping.host_port);
                 }
             }
        }

        Ok(Running {
            pid,
            started_at: time::OffsetDateTime::now_utc(),
        })
    }

    async fn stop(&self, container: &Container<Running>) -> Result<(), Error> {
        let pid = container.state().pid;
        
        // 1. Cleanup Port Forwarding
        for mapping in &container.config().port_mappings {
             info!("Cleaning up port forward: {}", mapping.host_port);
             let _ = std::process::Command::new("netsh")
                .arg("interface").arg("portproxy")
                .arg("delete").arg("v4tov4")
                .arg(format!("listenport={}", mapping.host_port))
                .arg("listenaddress=0.0.0.0")
                .status();
        }

        // 2. Stop process
        process_control::stop_container(pid)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(std::io::Error),
    #[error("Failed to get PID")]
    NoPid,
    #[error("Failed to setup logs: {0}")]
    LogSetupFailed(std::io::Error),
    #[error("Image resolution failed: {0}")]
    ImageResolutionFailed(String),
    #[error("RootFS composition failed: {0}")]
    RootfsCompositionFailed(String),
    #[error("Port forwarding failed: {0}")]
    PortForwardingFailed(String),
}

impl furukawa_common::diagnostic::Diagnosable for RuntimeError {
    fn code(&self) -> String {
        match self {
            Self::SpawnFailed(_) => "RUNTIME_SPAWN_FAILED".to_string(),
            Self::NoPid => "RUNTIME_NO_PID".to_string(),
            Self::LogSetupFailed(_) => "RUNTIME_LOG_SETUP_FAILED".to_string(),
            Self::ImageResolutionFailed(_) => "RUNTIME_IMAGE_RESOLUTION_FAILED".to_string(),
            Self::RootfsCompositionFailed(_) => "RUNTIME_ROOTFS_COMPOSITION_FAILED".to_string(),
            Self::PortForwardingFailed(_) => "RUNTIME_PORT_FORWARDING_FAILED".to_string(),
        }
    }
    fn suggestion(&self) -> Option<String> {
        match self {
            Self::SpawnFailed(_) => Some("Check if the binary exists and has execution permissions".to_string()),
            Self::NoPid => Some("Process might have exited immediately".to_string()),
            Self::LogSetupFailed(_) => Some("Check disk permissions".to_string()),
            Self::ImageResolutionFailed(_) => Some("Verify if the image exists in the store".to_string()),
            Self::RootfsCompositionFailed(_) => Some("Check for enough disk space and file permissions".to_string()),
            Self::PortForwardingFailed(_) => Some("Check if firewall or another process is blocking the port, and ensure Admin privileges".to_string()),
        }
    }
}
