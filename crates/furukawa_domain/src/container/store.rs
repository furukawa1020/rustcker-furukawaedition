use super::Container;
use async_trait::async_trait;
use furukawa_common::Result;

#[async_trait]
pub trait ContainerStore: Send + Sync {
    async fn save(&self, container: &Container<super::Created>) -> Result<()>;
    async fn list(&self) -> Result<Vec<Container<super::Created>>>;
    async fn get(&self, id: &str) -> Result<Option<Container<super::Created>>>; // Simplified generic for now
}
