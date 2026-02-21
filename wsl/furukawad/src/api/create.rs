use axum::{extract::{Query, Json, State}, response::IntoResponse, Json as AxumJson};
use furukawa_infra_docker::v1_45::{ContainerConfig, ContainerCreateResponse};
use serde::Deserialize;
use tracing::info;
use furukawa_domain::container::{Container, config::PortMapping};
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

    // 2. Initialize Domain FSM
    // In a real implementation:
    // - Check if image exists (ImageStore)
    // - Validate name uniqueness (ContainerStore)
    // - Create filesystem layer (COW)
    
    // STRICT: Initialize pure Rust domain object
    let mut port_mappings = Vec::new();
    if let Some(host_config) = &body.host_config {
        if let Some(bindings) = &host_config.port_bindings {
            for (container_port_proto, binding_list) in bindings {
                // container_port_proto is like "80/tcp"
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
    }

    let config = furukawa_domain::container::Config {
        image: body.image.clone(),
        cmd: body.cmd.clone().unwrap_or_default(),
        port_mappings,
    };
    let container = Container::new(id.clone(), config);
    
    // 3. Persist State (SQLite)
    // We unwrap here for prototype phase, but in production this maps to 500
    state.container_store.save(&container).await.unwrap();

    let resp = ContainerCreateResponse {
        id,
        warnings: vec![],
    };

    (axum::http::StatusCode::CREATED, AxumJson(resp))
}
