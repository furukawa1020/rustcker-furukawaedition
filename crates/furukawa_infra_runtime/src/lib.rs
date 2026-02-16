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

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(std::io::Error),
    #[error("Failed to get PID")]
    NoPid,
    #[error("Failed to setup logs: {0}")]
    LogSetupFailed(std::io::Error),
}

impl furukawa_common::diagnostic::Diagnosable for RuntimeError {
    fn code(&self) -> String {
        match self {
            Self::SpawnFailed(_) => "RUNTIME_SPAWN_FAILED".to_string(),
            Self::NoPid => "RUNTIME_NO_PID".to_string(),
            Self::LogSetupFailed(_) => "RUNTIME_LOG_SETUP_FAILED".to_string(),
        }
    }
    fn suggestion(&self) -> Option<String> {
        match self {
            Self::SpawnFailed(_) => Some("Check if the binary exists and has execution permissions".to_string()),
            Self::NoPid => Some("Process might have exited immediately".to_string()),
            Self::LogSetupFailed(_) => Some("Check disk permissions".to_string()),
        }
    }
}
