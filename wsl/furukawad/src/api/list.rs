use axum::{extract::{Query, State}, response::IntoResponse, Json as AxumJson};
use furukawa_infra_docker::v1_45::{ContainerSummary, HostConfigSummary, SummaryNetworkSettings};
use serde::Deserialize;
use tracing::info;
use crate::state::AppState;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ListQueryParams {
    all: Option<bool>,
    limit: Option<isize>,
    size: Option<bool>,
    filters: Option<String>,
}

pub async fn handle(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> impl IntoResponse {
    info!(
        all = ?params.all,
        limit = ?params.limit,
        size = ?params.size,
        filters = ?params.filters,
        "Received container list request"
    );

    // Fetch from domain store
    // In production, we would map domain filters to store queries
    let containers = state.container_store.list().await.unwrap_or_else(|e| {
        tracing::error!("Failed to list containers: {}", e);
        vec![]
    });

    // Map Domain Container -> API ContainerSummary
    let summary: Vec<ContainerSummary> = containers.into_iter().map(|c| {
        ContainerSummary {
            id: c.id().to_string(),
            names: vec![format!("/furukawa-{}", &c.id()[0..8])], 
            image: c.config().image.clone(),
            image_id: "sha256:placeholder".to_string(),
            command: c.config().cmd.join(" "),
            created: 1700000000, // Placeholder timestamp
            ports: vec![],
            labels: HashMap::new(),
            state: "created".to_string(),
            status: "Created".to_string(),
            host_config: HostConfigSummary {
                network_mode: "default".to_string(),
            },
            network_settings: SummaryNetworkSettings {
                networks: HashMap::new(),
            },
            mounts: vec![],
        }
    }).collect();

    AxumJson(summary)
}
