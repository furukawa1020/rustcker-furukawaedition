use axum::{extract::{Query, Json, State}, response::IntoResponse, Json as AxumJson};
use furukawa_infra_docker::v1_45::{ContainerConfig, ContainerCreateResponse};
use serde::Deserialize;
use tracing::info;
use furukawa_domain::container::{Container, config::{PortMapping, VolumeMount}};
use crate::state::AppState;

use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateQueryParams {
    name: Option<String>,
    platform: Option<String>,
}

pub async fn handle(
    State(state): State<AppState>,
    Query(params): Query<CreateQueryParams>,
    Json(body): Json<ContainerConfig>,
) -> impl IntoResponse {
    info!(
        image = %body.image,
        name = ?params.name,
        platform = ?params.platform,
        "Received container creation request"
    );

    // 1. Generate ID (Strict UUID v4)
    let id = Uuid::new_v4().to_string();

    // 2. Parse HostConfig: port bindings, volume binds, network mode
    let mut port_mappings = Vec::new();
    let mut volumes = Vec::new();
    let mut network = "bridge".to_string();

    if let Some(host_config) = &body.host_config {
        // Port mappings
        if let Some(bindings) = &host_config.port_bindings {
            for (container_port_proto, binding_list) in bindings {
                let parts: Vec<&str> = container_port_proto.split('/').collect();
                let container_port = parts[0].parse::<u16>().unwrap_or(0);
                let protocol = parts.get(1).unwrap_or(&"tcp").to_string();

                for binding in binding_list {
                    if let Some(host_port_str) = &binding.host_port {
                        if let Ok(host_port) = host_port_str.parse::<u16>() {
                            port_mappings.push(PortMapping {
                                container_port,
                                host_port,
                                protocol: protocol.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Volume bind mounts: "C:\data:/data" or "C:\data:/data:ro"
        if let Some(binds) = &host_config.binds {
            for bind in binds {
                let parts: Vec<&str> = bind.splitn(3, ':').collect();
                if parts.len() >= 2 {
                    let readonly = parts.get(2).map(|f| *f == "ro").unwrap_or(false);
                    volumes.push(VolumeMount {
                        host_path: parts[0].to_string(),
                        container_path: parts[1].to_string(),
                        readonly,
                    });
                }
            }
        }

        // Network mode
        if let Some(nm) = &host_config.network_mode {
            network = nm.clone();
        }
    }

    // 3. Parse Env: ["KEY=VALUE", ...]
    let env = body.env.unwrap_or_default();

    let config = furukawa_domain::container::Config {
        image: body.image.clone(),
        cmd: body.cmd.clone().unwrap_or_default(),
        port_mappings,
        volumes,
        env,
        network,
    };
    let container = Container::new(id.clone(), config);
    
    // 4. Persist State (SQLite)
    state.container_store.save(&container).await.unwrap();

    let resp = ContainerCreateResponse {
        id,
        warnings: vec![],
    };

    (axum::http::StatusCode::CREATED, AxumJson(resp))
}
