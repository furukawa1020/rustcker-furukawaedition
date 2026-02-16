use axum::{extract::{Path, State}, response::IntoResponse, http::StatusCode, Json};
use serde::Serialize;
use serde_json::json;
use crate::state::AppState;
use furukawa_domain::container::AnyContainer;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerInspect {
    pub id: String,
    pub created:String,
    pub path: String,
    pub args: Vec<String>,
    pub state: ContainerState,
    pub image: String,
    pub name: String,
    pub restart_count: u32,
    pub driver: String,
    pub platform: String,
    pub mount_label: String,
    pub process_label: String,
    pub app_armor_profile: String,
    pub exec_i_ds: Option<Vec<String>>,
    pub host_config: serde_json::Value,
    pub graph_driver: serde_json::Value,
    pub size_rw: Option<i64>,
    pub size_root_fs: Option<i64>,
    pub config: serde_json::Value,
    pub network_settings: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerState {
    pub status: String,
    pub running: bool,
    pub paused: bool,
    pub restarting: bool,
    pub o_o_m_killed: bool,
    pub dead: bool,
    pub pid: u32,
    pub exit_code: i32,
    pub error: String,
    pub started_at: String,
    pub finished_at: String,
}

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.container_store.get_any(&id).await {
        Ok(Some(container)) => {
            let config = container.config();
            let mut path = "".to_string();
            let mut args = vec![];
            
            if !config.cmd.is_empty() {
                path = config.cmd[0].clone();
                if config.cmd.len() > 1 {
                    args = config.cmd[1..].to_vec();
                }
            }
            
            let status = container.status().to_string();
            let running = status == "running";
            
            let (pid, started_at, finished_at, exit_code) = match &container {
                AnyContainer::Running(c) => (c.state().pid, c.state().started_at.to_string(), "0001-01-01T00:00:00Z".to_string(), 0),
                AnyContainer::Stopped(c) => (0, "0001-01-01T00:00:00Z".to_string(), c.state().finished_at.to_string(), c.state().exit_code),
                _ => (0, "0001-01-01T00:00:00Z".to_string(), "0001-01-01T00:00:00Z".to_string(), 0),
            };

            let inspect = ContainerInspect {
                id: container.id().to_string(),
                created: "2024-01-01T00:00:00Z".to_string(), // TODO: persist created_at
                path,
                args,
                state: ContainerState {
                    status,
                    running,
                    paused: false,
                    restarting: false,
                    o_o_m_killed: false,
                    dead: false,
                    pid,
                    exit_code,
                    error: "".to_string(),
                    started_at,
                    finished_at,
                },
                image: config.image.clone(),
                name: format!("/{}", container.id()), // TODO: Implement naming
                restart_count: 0,
                driver: "furukawa-fs".to_string(),
                platform: "windows".to_string(),
                mount_label: "".to_string(),
                process_label: "".to_string(),
                app_armor_profile: "".to_string(),
                exec_i_ds: None,
                host_config: json!({}),
                graph_driver: json!({}),
                size_rw: None,
                size_root_fs: None,
                config: json!(config),
                network_settings: json!({
                    "Bridge": "",
                    "SandboxID": "",
                    "HairpinMode": false,
                    "LinkLocalIPv6Address": "",
                    "LinkLocalIPv6PrefixLen": 0,
                    "Ports": {},
                    "SandboxKey": "",
                    "SecondaryIPAddresses": null,
                    "SecondaryIPv6Addresses": null,
                    "EndpointID": "",
                    "Gateway": "",
                    "GlobalIPv6Address": "",
                    "GlobalIPv6PrefixLen": 0,
                    "IPAddress": "",
                    "IPPrefixLen": 0,
                    "IPv6Gateway": "",
                    "MacAddress": "",
                    "Networks": {}
                }),
            };

            (StatusCode::OK, Json(inspect)).into_response()
        },
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
