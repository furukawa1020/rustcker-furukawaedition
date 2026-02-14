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

pub struct Container<S> {
    id: String,
    state: S,
}

// States
pub struct Created;
pub struct Running;
pub struct Stopped;

impl Container<Created> {
    pub fn new(id: String) -> self::Container<Created> {
        Container {
            id,
            state: Created,
        }
    }

    pub fn start(self) -> Result<Container<Running>, Error> {
        // Validation logic would go here
        Ok(Container {
            id: self.id,
            state: Running,
        })
    }
}

impl Container<Running> {
    pub fn stop(self) -> Result<Container<Stopped>, Error> {
        Ok(Container {
            id: self.id,
            state: Stopped,
        })
    }
}
