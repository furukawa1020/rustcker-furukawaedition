use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageSummary {
    pub id: String,
    pub parent_id: String,
    pub repo_tags: Option<Vec<String>>,
    pub repo_digests: Option<Vec<String>>,
    pub created: i64,
    pub size: i64,
    pub shared_size: i64,
    pub virtual_size: i64,
    pub labels: Option<HashMap<String, String>>,
    pub containers: i64,
}
