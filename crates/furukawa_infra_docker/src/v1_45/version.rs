use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Version {
    pub platform: Platform,
    pub components: Option<Vec<Component>>,
    pub version: String,
    pub api_version: String,
    pub min_a_p_i_version: String,
    pub git_commit: String,
    pub go_version: String,
    pub os: String,
    pub arch: String,
    pub kernel_version: String,
    pub experimental: bool,
    pub build_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Platform {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Component {
    pub name: String,
    pub version: String,
    pub details: Option<serde_json::Value>,
}
