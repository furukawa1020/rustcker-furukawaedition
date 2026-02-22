use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub image: String,
    pub cmd: Vec<String>,
    pub port_mappings: Vec<PortMapping>,
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub network: String, // e.g. "bridge", "host", "none", or custom name
}

impl Default for Config {
    fn default() -> Self {
        Self {
            image: String::new(),
            cmd: Vec::new(),
            port_mappings: Vec::new(),
            volumes: Vec::new(),
            env: Vec::new(),
            network: "bridge".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: String, // "tcp" or "udp"
}

/// A volume bind-mount request: host path -> container path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Absolute Windows host path (e.g. "C:\\data")
    pub host_path: String,
    /// Absolute container path (e.g. "/data")
    pub container_path: String,
    /// If true, mount is read-only
    pub readonly: bool,
}
