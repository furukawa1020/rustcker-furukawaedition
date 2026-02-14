use axum::{extract::{Query, Json, State}, response::IntoResponse, Json as AxumJson};
use furukawa_infra_docker::v1_45::{ContainerConfig, ContainerCreateResponse};
use serde::Deserialize;
use tracing::info;
use furukawa_domain::container::Container;
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
    let config = furukawa_domain::container::Config {
        image: body.image.clone(),
        cmd: body.cmd.clone().unwrap_or_default(),
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
