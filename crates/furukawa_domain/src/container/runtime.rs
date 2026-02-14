use super::{Container, Created, Running};
use async_trait::async_trait;
use furukawa_common::Result;

#[async_trait]
pub trait ContainerRuntime: Send + Sync {
    /// Starts a container.
    /// This is a low-level operation that spawns the process.
    /// It returns the new `Running` state which includes the PID.
    async fn start(&self, container: &Container<Created>) -> Result<Container<Running>>;
    
    // Future: stop, kill, pause, etc.
}
