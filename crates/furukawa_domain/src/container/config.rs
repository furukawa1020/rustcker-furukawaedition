use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub image: String,
    pub cmd: Vec<String>,
    pub port_mappings: Vec<PortMapping>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: String, // "tcp" or "udp"
}
