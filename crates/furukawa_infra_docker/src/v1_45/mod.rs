use serde::{Deserialize, Serialize};

// Per spec: Unknown fields should be ignored.
// We do NOT use #[serde(deny_unknown_fields)]

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerConfig {
    pub hostname: Option<String>,
    pub domainname: Option<String>,
    pub user: Option<String>,
    pub image: String,
}
