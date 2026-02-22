use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::state::AppState;
use tracing::info;

/// Docker Engine API v1.45 — Network object
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Network {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub enable_i_pv6: bool,
    pub internal: bool,
    pub attachable: bool,
    pub ingress: bool,
    pub labels: HashMap<String, String>,
}

impl Network {
    fn bridge() -> Self {
        Self {
            id: "bridge".to_string(),
            name: "bridge".to_string(),
            driver: "bridge".to_string(),
            scope: "local".to_string(),
            enable_i_pv6: false,
            internal: false,
            attachable: false,
            ingress: false,
            labels: HashMap::new(),
        }
    }
    fn host() -> Self {
        Self {
            id: "host".repeat(2),
            name: "host".to_string(),
            driver: "host".to_string(),
            scope: "local".to_string(),
            enable_i_pv6: false,
            internal: false,
            attachable: false,
            ingress: false,
            labels: HashMap::new(),
        }
    }
    fn none() -> Self {
        Self {
            id: "none".repeat(2),
            name: "none".to_string(),
            driver: "null".to_string(),
            scope: "local".to_string(),
            enable_i_pv6: false,
            internal: false,
            attachable: false,
            ingress: false,
            labels: HashMap::new(),
        }
    }
}

/// GET /networks — List all networks
pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let mut networks = vec![Network::bridge(), Network::host(), Network::none()];

    // Load custom networks from store
    if let Ok(custom) = state.network_store.list().await {
        networks.extend(custom);
    }

    Json(networks)
}

/// GET /networks/{id} — Inspect a network
pub async fn inspect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Built-in networks
    let builtin = match id.as_str() {
        "bridge" => Some(Network::bridge()),
        "host" => Some(Network::host()),
        "none" => Some(Network::none()),
        _ => None,
    };

    if let Some(net) = builtin {
        return (axum::http::StatusCode::OK, Json(serde_json::to_value(net).unwrap()));
    }

    // Custom network from store
    match state.network_store.get(&id).await {
        Ok(Some(net)) => (axum::http::StatusCode::OK, Json(serde_json::to_value(net).unwrap())),
        _ => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("network {} not found", id) })),
        ),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateNetworkRequest {
    pub name: String,
    pub driver: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateNetworkResponse {
    pub id: String,
}

/// POST /networks/create — Create a custom network
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateNetworkRequest>,
) -> impl IntoResponse {
    use uuid::Uuid;
    let id = Uuid::new_v4().to_string();
    info!("Creating network '{}' with driver '{}'", body.name, body.driver.as_deref().unwrap_or("bridge"));

    let net = Network {
        id: id.clone(),
        name: body.name,
        driver: body.driver.unwrap_or_else(|| "bridge".to_string()),
        scope: "local".to_string(),
        enable_i_pv6: false,
        internal: false,
        attachable: false,
        ingress: false,
        labels: body.labels.unwrap_or_default(),
    };

    if let Err(e) = state.network_store.save(&net).await {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        );
    }

    (axum::http::StatusCode::CREATED, Json(serde_json::to_value(CreateNetworkResponse { id }).unwrap()))
}

/// DELETE /networks/{id} — Remove a network
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Built-in networks can't be deleted
    if matches!(id.as_str(), "bridge" | "host" | "none") {
        return axum::http::StatusCode::FORBIDDEN;
    }

    match state.network_store.delete(&id).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(_) => axum::http::StatusCode::NOT_FOUND,
    }
}
