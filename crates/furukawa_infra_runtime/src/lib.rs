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

        // Strict: Use Command::new
        let child = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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
}

impl furukawa_common::diagnostic::Diagnosable for RuntimeError {
    fn code(&self) -> String {
        match self {
            Self::SpawnFailed(_) => "RUNTIME_SPAWN_FAILED".to_string(),
            Self::NoPid => "RUNTIME_NO_PID".to_string(),
        }
    }
    fn suggestion(&self) -> Option<String> {
        match self {
            Self::SpawnFailed(_) => Some("Check if the binary exists and has execution permissions".to_string()),
            Self::NoPid => Some("Process might have exited immediately".to_string()),
        }
    }
}
