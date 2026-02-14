use axum::{extract::{Query, Json}, response::IntoResponse, Json as AxumJson};
use furukawa_infra_docker::v1_45::{ContainerConfig, ContainerCreateResponse};
use serde::Deserialize;
use tracing::info;
use furukawa_domain::container::{Container, Created};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateQueryParams {
    name: Option<String>,
    platform: Option<String>,
}

pub async fn handle(
    Query(params): Query<CreateQueryParams>,
    Json(body): Json<ContainerConfig>,
) -> impl IntoResponse {
    info!(
        image = %body.image,
        name = ?params.name,
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
    let _container = Container::new(id.clone());
    
    // 3. Persist State (TODO: SQLite)
    // For now, we simulate success.

    let resp = ContainerCreateResponse {
        id,
        warnings: vec![],
    };

    (axum::http::StatusCode::CREATED, AxumJson(resp))
}
