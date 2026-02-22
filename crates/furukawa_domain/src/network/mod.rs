//! Network store trait – abstraction over persistence for Docker networks.

use async_trait::async_trait;
use crate::Result;
use serde::{Deserialize, Serialize};

/// Represents a persisted Docker-compatible network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRecord {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub labels: std::collections::HashMap<String, String>,
}

#[async_trait]
pub trait NetworkStore: Send + Sync {
    async fn save(&self, network: &NetworkRecord) -> Result<()>;
    async fn list(&self) -> Result<Vec<NetworkRecord>>;
    async fn get(&self, id: &str) -> Result<Option<NetworkRecord>>;
    async fn delete(&self, id: &str) -> Result<()>;
}
