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

        let stdout_file = log_file.try_clone().map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        let stderr_file = log_file;

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

/// Convert a Windows absolute path to a WSL path using `wslpath -u`.
/// Falls back to naive `/mnt/c/...` conversion if wslpath is unavailable.
async fn windows_to_wsl_path(distro: &str, windows_path: &str) -> String {
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", distro, "--", "wslpath", "-u", windows_path])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !path.is_empty() {
                return path;
            }
        }
        _ => {}
    }

    // Fallback: naive C:\ -> /mnt/c/
    let normalized = windows_path.replace('\\', "/");
    if let Some(stripped) = normalized.strip_prefix("C:/").or_else(|| normalized.strip_prefix("c:/")) {
        format!("/mnt/c/{}", stripped)
    } else {
        normalized
    }
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

        // 3. Convert Windows rootfs path to WSL path
        let wsl_rootfs = windows_to_wsl_path(
            &self.distro,
            rootfs_dir.to_str().unwrap_or(""),
        ).await;

        // 4. Set up volume bind-mounts: create mount points inside rootfs, then bind
        for vol in &config.volumes {
            let wsl_host = windows_to_wsl_path(&self.distro, &vol.host_path).await;
            let mountpoint = format!("{}{}", wsl_rootfs, vol.container_path);

            // Create mountpoint inside rootfs
            let _ = std::process::Command::new("wsl.exe")
                .args(["-d", &self.distro, "-u", "root", "--", "mkdir", "-p", &mountpoint])
                .status();

            // Bind mount: mount --bind <host_wsl_path> <rootfs><container_path>
            let mut bind_args = vec!["mount", "--bind"];
            if vol.readonly {
                bind_args.push("-o");
                bind_args.push("ro");
            }
            // We'll pass this as a pre-start command via WSL
            let status = std::process::Command::new("wsl.exe")
                .args(["-d", &self.distro, "-u", "root", "--", "mount", "--bind"])
                .arg(&wsl_host)
                .arg(&mountpoint)
                .status();

            match status {
                Ok(s) if !s.success() => {
                    tracing::warn!("Failed to bind-mount {} -> {}", vol.host_path, vol.container_path);
                }
                Err(e) => {
                    tracing::warn!("mount --bind error: {}", e);
                }
                _ => {
                    info!("Bind-mounted {} -> {}", vol.host_path, vol.container_path);
                }
            }
        }

        // 5. Build the WSL command
        let program = if config.cmd.is_empty() { "sh" } else { &config.cmd[0] };
        let args = if config.cmd.len() > 1 { &config.cmd[1..] } else { &[] };

        // Start with env vars via `env KEY=VALUE ... chroot <rootfs> <program>`
        let mut wsl_cmd = Command::new("wsl.exe");
        wsl_cmd.arg("-d").arg(&self.distro)
               .arg("-u").arg("root")
               .arg("--")
               .arg("env");
        
        // Forward environment variables
        for env_var in &config.env {
            wsl_cmd.arg(env_var);
        }

        wsl_cmd.arg("chroot").arg(&wsl_rootfs).arg(program);
        for arg in args {
            wsl_cmd.arg(arg);
        }

        // 6. Setup container logs
        let log_dir = std::path::Path::new("furukawa_logs");
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir).map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        }
        let log_path = log_dir.join(format!("{}.log", container.id()));
        let log_file = std::fs::File::create(&log_path).map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        let stdout_file = log_file.try_clone().map_err(|e| Error::new(RuntimeError::LogSetupFailed(e)))?;
        let stderr_file = log_file;

        info!("Spawning WSL process: env {:?} chroot {} {}", config.env, wsl_rootfs, program);
        let child = wsl_cmd
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|e| Error::new(RuntimeError::SpawnFailed(e)))?;

        let pid = child.id().ok_or_else(|| Error::new(RuntimeError::NoPid))?;
        
        // 7. Port Forwarding (Netsh)
        if !config.port_mappings.is_empty() {
             let output = std::process::Command::new("wsl.exe")
                .arg("-d").arg(&self.distro)
                .arg("hostname").arg("-I")
                .output()
                .map_err(|e| Error::new(RuntimeError::PortForwardingFailed(format!("Failed to get WSL IP: {}", e))))?;
             
             let ip_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
             let wsl_ip = ip_str.split_whitespace().next().ok_or_else(|| Error::new(RuntimeError::PortForwardingFailed("No WSL IP found".into())))?;

             for mapping in &config.port_mappings {
                 info!("Setting up port forward: {} -> {}:{}", mapping.host_port, wsl_ip, mapping.container_port);
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
        
        // 1. Unmount bind-mounts
        for vol in &container.config().volumes {
            let container_dir = self.containers_root.join(container.id());
            let rootfs_dir = container_dir.join("rootfs");
            let mountpoint = format!("{}{}", rootfs_dir.to_string_lossy().replace('\\', "/"), vol.container_path);
            let _ = std::process::Command::new("wsl.exe")
                .args(["-d", &self.distro, "-u", "root", "--", "umount", &mountpoint])
                .status();
        }

        // 2. Cleanup Port Forwarding
        for mapping in &container.config().port_mappings {
             info!("Cleaning up port forward: {}", mapping.host_port);
             let _ = std::process::Command::new("netsh")
                .arg("interface").arg("portproxy")
                .arg("delete").arg("v4tov4")
                .arg(format!("listenport={}", mapping.host_port))
                .arg("listenaddress=0.0.0.0")
                .status();
        }

        // 3. Stop process
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
    #[error("WSL setup failed: {0}")]
    WslSetupFailed(String),
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
            Self::WslSetupFailed(_) => "RUNTIME_WSL_SETUP_FAILED".to_string(),
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
            Self::WslSetupFailed(_) => Some("Ensure WSL2 is installed (wsl --install) and enabled".to_string()),
        }
    }
}
