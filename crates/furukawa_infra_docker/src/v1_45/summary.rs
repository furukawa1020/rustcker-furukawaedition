use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Implements Docker Engine API v1.45 ContainerSummary
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerSummary {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub image_id: String,
    pub command: String,
    pub created: i64,
    pub ports: Vec<Port>,
    pub labels: HashMap<String, String>,
    pub state: String,
    pub status: String,
    pub host_config: HostConfigSummary,
    pub network_settings: SummaryNetworkSettings,
    pub mounts: Vec<Mount>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Port {
    #[serde(rename = "IP")]
    pub ip: Option<String>,
    pub private_port: u16,
    pub public_port: Option<u16>,
    #[serde(rename = "Type")]
    pub type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HostConfigSummary {
    pub network_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SummaryNetworkSettings {
    pub networks: HashMap<String, NetworkEndpointSettings>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkEndpointSettings {
    #[serde(rename = "IPAMConfig")]
    pub ipam_config: Option<EndpointIpamConfig>,
    pub links: Option<Vec<String>>,
    pub aliases: Option<Vec<String>>,
    #[serde(rename = "NetworkID")]
    pub network_id: String,
    #[serde(rename = "EndpointID")]
    pub endpoint_id: String,
    #[serde(rename = "Gateway")]
    pub gateway: String,
    #[serde(rename = "IPAddress")]
    pub ip_address: String,
    #[serde(rename = "IPPrefixLen")]
    pub ip_prefix_len: i64,
    #[serde(rename = "IPv6Gateway")]
    pub ipv6_gateway: String,
    #[serde(rename = "GlobalIPv6Address")]
    pub global_ipv6_address: String,
    #[serde(rename = "GlobalIPv6PrefixLen")]
    pub global_ipv6_prefix_len: i64,
    #[serde(rename = "MacAddress")]
    pub mac_address: String,
    pub driver_opts: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EndpointIpamConfig {
    #[serde(rename = "IPv4Address")]
    pub ipv4_address: Option<String>,
    #[serde(rename = "IPv6Address")]
    pub ipv6_address: Option<String>,
    pub link_local_ips: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Mount {
    pub name: Option<String>,
    pub source: String,
    pub destination: String,
    pub driver: Option<String>,
    pub mode: String,
    #[serde(rename = "RW")]
    pub rw: bool,
    pub propagation: String,
    #[serde(rename = "Type")]
    pub type_: String,
}
