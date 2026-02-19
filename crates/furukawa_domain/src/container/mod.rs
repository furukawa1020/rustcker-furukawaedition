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
pub mod state_serde;

#[derive(Debug, Clone, PartialEq)]
pub struct Container<S> {
    id: String,
    config: Config,
    state: S,
}

#[derive(Debug, Clone)]
pub enum AnyContainer {
    Created(Container<Created>),
    Running(Container<Running>),
    Stopped(Container<Stopped>),
}

impl AnyContainer {
    pub fn id(&self) -> &str {
        match self {
            Self::Created(c) => c.id(),
            Self::Running(c) => c.id(),
            Self::Stopped(c) => c.id(),
        }
    }

    pub fn config(&self) -> &Config {
        match self {
            Self::Created(c) => c.config(),
            Self::Running(c) => c.config(),
            Self::Stopped(c) => c.config(),
        }
    }

    pub fn status(&self) -> &'static str {
        match self {
            Self::Created(_) => "created",
            Self::Running(_) => "running",
            Self::Stopped(_) => "exited",
        }
    }
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
#[derive(Debug, PartialEq, Clone)]
pub struct Created;

#[derive(Debug, PartialEq, Clone)]
pub struct Running {
    pub pid: u32,
    pub started_at: time::OffsetDateTime,
}

#[derive(Debug, PartialEq, Clone)]
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
}

impl Container<Running> {
    pub fn restore(id: String, config: Config, state: Running) -> self::Container<Running> {
        Container {
            id,
            config,
            state,
        }
    }

    pub async fn stop(self, runtime: &(impl runtime::ContainerRuntime + ?Sized)) -> Result<Container<Stopped>, Error> {
        runtime.stop(&self).await?;
        
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


impl Container<Created> {
    pub async fn start(self, runtime: &(impl runtime::ContainerRuntime + ?Sized)) -> Result<Container<Running>, Error> {
        let running_state = runtime.start(&self).await?;
        Ok(Container {
            id: self.id,
            config: self.config,
            state: running_state,
        })
    }
}

impl Container<Stopped> {
    pub fn restore(id: String, config: Config, state: Stopped) -> self::Container<Stopped> {
        Container {
            id,
            config,
            state,
        }
    }
}

#[cfg(test)]
mod tests;
pub mod store;
pub mod runtime;


