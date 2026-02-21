use async_trait::async_trait;
use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub parent_id: Option<String>,
    pub created: i64, // Unix timestamp
    pub size: i64,
    pub layers: Vec<String>,
}

#[async_trait]
pub trait ImageMetadataStore: Send + Sync {
    async fn save(&self, metadata: &ImageMetadata) -> Result<()>;
    async fn list(&self) -> Result<Vec<ImageMetadata>>;
    async fn get(&self, id: &str) -> Result<Option<ImageMetadata>>;
    async fn exists(&self, id: &str) -> Result<bool>;
}
