use super::Container;
use async_trait::async_trait;
use furukawa_common::Result;

#[async_trait]
pub trait ContainerStore: Send + Sync {
    async fn save(&self, container: &Container<super::Created>) -> Result<()>;
    async fn save_running(&self, container: &Container<super::Running>) -> Result<()>;
    async fn save_stopped(&self, container: &Container<super::Stopped>) -> Result<()>;
    async fn list(&self) -> Result<Vec<Container<super::Created>>>;
    async fn get(&self, id: &str) -> Result<Option<Container<super::Created>>>; 
    async fn get_running(&self, id: &str) -> Result<Option<Container<super::Running>>>;
}
