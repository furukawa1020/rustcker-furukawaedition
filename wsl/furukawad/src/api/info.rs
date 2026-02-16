use axum::{extract::State, response::IntoResponse, Json, http::StatusCode};
use furukawa_infra_docker::v1_45::{SystemInfo, PluginsInfo};
use furukawa_domain::container::AnyContainer;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let containers = match state.container_store.list().await {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let total = containers.len();
    let running = containers.iter().filter(|c| matches!(c, AnyContainer::Running(_))).count();
    let stopped = containers.iter().filter(|c| matches!(c, AnyContainer::Stopped(_))).count();
    let paused = 0; // Not implemented yet

    // Get Memory Total from /proc/meminfo (simple parsing)
    let mem_total = if let Ok(info) = std::fs::read_to_string("/proc/meminfo") {
        info.lines()
            .find(|l| l.starts_with("MemTotal:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0) * 1024 // kB to Bytes
    } else {
        0
    };

    let n_cpu = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    let info = SystemInfo {
        id: "furukawa-engine-id".to_string(), // TODO: Persistent ID
        containers: total,
        containers_running: running,
        containers_paused: paused,
        containers_stopped: stopped,
        images: 0,
        driver: "furukawa-fs".to_string(),
        system_status: None,
        plugins: PluginsInfo {
            volume: Some(vec!["local".to_string()]),
            network: Some(vec!["bridge".to_string(), "host".to_string()]),
            authorization: None,
            log: Some(vec!["json-file".to_string()]),
        },
        memory_limit: true,
        swap_limit: true,
        kernel_memory: true,
        kernel_memory_t_c_p: true,
        cpu_cfs_period: true,
        cpu_cfs_quota: true,
        c_p_u_shares: true,
        c_p_u_set: true,
        pids_limit: true,
        oom_kill_disable: true,
        i_pv4_forwarding: true,
        bridge_nf_iptables: true,
        bridge_nf_ip6tables: true,
        debug: true,
        n_fd: 0,
        o_o_m_score_adj: 0,
        n_e_vents_listener: 0,
        kernel_version: "5.15.90.1-microsoft-standard-WSL2".to_string(), // TODO: uname -r
        operating_system: "Docker Desktop".to_string(), // Client expects this for context
        o_s_type: "linux".to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        n_c_p_u: n_cpu,
        mem_total,
        name: "furukawa-engine".to_string(),
        server_version: "0.1.0".to_string(),
    };

    Json(info).into_response()
}
