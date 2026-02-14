use furukawa_common::diagnostic::{Diagnosable, Error};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("Invalid transition")]
    InvalidTransition,
}

impl Diagnosable for ContainerError {
    fn code(&self) -> String {
        "CONTAINER_INVALID_TRANSITION".to_string()
    }
    fn suggestion(&self) -> Option<String> {
        Some("Check container state before operation".to_string())
    }
}

pub mod config;
pub use config::Config;

#[derive(Debug, Clone, PartialEq)]
pub struct Container<S> {
    id: String,
    config: Config,
    state: S,
}

impl<S> Container<S> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn state(&self) -> &S {
        &self.state
    }
}


// States
#[derive(Debug, PartialEq)]
pub struct Created;

#[derive(Debug, PartialEq)]
pub struct Running {
    pub pid: u32,
    pub started_at: time::OffsetDateTime,
}

#[derive(Debug, PartialEq)]
pub struct Stopped {
    pub finished_at: time::OffsetDateTime,
    pub exit_code: i32,
}

impl Container<Created> {
    pub fn new(id: String, config: Config) -> self::Container<Created> {
        Container {
            id,
            config,
            state: Created,
        }
    }

    pub async fn start(self, runtime: &impl runtime::ContainerRuntime) -> Result<Container<Running>, Error> {
        let running_state = runtime.start(&self).await?;
        Ok(Container {
            id: self.id,
            config: self.config,
            state: running_state,
        })
    }
}

impl Container<Running> {
    pub async fn stop(self, runtime: &impl runtime::ContainerRuntime) -> Result<Container<Stopped>, Error> {
        // In a real implementation, we'd runtime.stop(&self)
        // For now, just return state
        Ok(Container {
            id: self.id,
            config: self.config,
            state: Stopped {
                finished_at: time::OffsetDateTime::now_utc(),
                exit_code: 0,
            },
        })
    }
}

mod tests;
pub mod store;
pub mod runtime;


