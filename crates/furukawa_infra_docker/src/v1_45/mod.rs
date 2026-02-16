use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Implements Docker Engine API v1.45 ContainerConfig
// Ref: https://docs.docker.com/engine/api/v1.45/#tag/Container/operation/ContainerCreate
// Constraint: Unknown fields are ignored by default in serde, which matches API spec behavior.

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerConfig {
    pub hostname: Option<String>,
    pub domainname: Option<String>,
    pub user: Option<String>,
    pub attach_stdin: Option<bool>,
    pub attach_stdout: Option<bool>,
    pub attach_stderr: Option<bool>,
    pub tty: Option<bool>,
    pub open_stdin: Option<bool>,
    pub stdin_once: Option<bool>,
    pub env: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub image: String, // Mandatory
    pub volumes: Option<HashMap<String, HashMap<(), ()>>>,
    pub working_dir: Option<String>,
    pub entrypoint: Option<Vec<String>>,
    pub network_disabled: Option<bool>,
    pub mac_address: Option<String>,
    pub on_build: Option<Vec<String>>,
    pub labels: Option<HashMap<String, String>>,
    pub stop_signal: Option<String>,
    pub stop_timeout: Option<isize>,
    pub shell: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerCreateResponse {
    pub id: String,
    pub warnings: Vec<String>,
}

mod summary;
pub use summary::*;

mod version;
pub use version::*;
