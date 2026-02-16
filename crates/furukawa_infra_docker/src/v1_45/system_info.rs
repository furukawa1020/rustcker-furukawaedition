use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemInfo {
    pub id: String,
    pub containers: usize,
    pub containers_running: usize,
    pub containers_paused: usize,
    pub containers_stopped: usize,
    pub images: usize,
    pub driver: String,
    pub system_status: Option<Vec<String>>,
    pub plugins: PluginsInfo,
    pub memory_limit: bool,
    pub swap_limit: bool,
    pub kernel_memory: bool,
    pub kernel_memory_t_c_p: bool,
    pub cpu_cfs_period: bool,
    pub cpu_cfs_quota: bool,
    pub c_p_u_shares: bool,
    pub c_p_u_set: bool,
    pub pids_limit: bool,
    pub oom_kill_disable: bool,
    pub i_pv4_forwarding: bool,
    pub bridge_nf_iptables: bool,
    pub bridge_nf_ip6tables: bool,
    pub debug: bool,
    pub n_fd: usize,
    pub o_o_m_score_adj: usize,
    pub n_e_vents_listener: usize,
    pub kernel_version: String,
    pub operating_system: String,
    pub o_s_type: String,
    pub architecture: String,
    pub n_c_p_u: usize,
    pub mem_total: i64,
    pub name: String,
    pub server_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PluginsInfo {
    pub volume: Option<Vec<String>>,
    pub network: Option<Vec<String>>,
    pub authorization: Option<Vec<String>>,
    pub log: Option<Vec<String>>,
}
