use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageRef {
    pub registry: Option<String>,
    pub repository: String,
    pub tag: String,
}

impl ImageRef {
    pub fn new(repository: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            registry: None,
            repository: repository.into(),
            tag: tag.into(),
        }
    }
}
