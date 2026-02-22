use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::state::AppState;
use furukawa_domain::network::NetworkRecord;
use tracing::info;
use uuid::Uuid;

/// Docker Engine API v1.45 — Network object (response shape)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkResponse {
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

fn builtin_network(id: &str, name: &str, driver: &str) -> NetworkResponse {
    NetworkResponse {
        id: id.to_string(),
        name: name.to_string(),
        driver: driver.to_string(),
        scope: "local".to_string(),
        enable_i_pv6: false,
        internal: false,
        attachable: false,
        ingress: false,
        labels: HashMap::new(),
    }
}

fn record_to_response(r: NetworkRecord) -> NetworkResponse {
    NetworkResponse {
        id: r.id,
        name: r.name,
        driver: r.driver,
        scope: "local".to_string(),
        enable_i_pv6: false,
        internal: false,
        attachable: false,
        ingress: false,
        labels: r.labels,
    }
}

/// GET /networks — List all networks
pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let mut networks = vec![
        builtin_network("bridge", "bridge", "bridge"),
        builtin_network("host0host0", "host", "host"),
        builtin_network("none0none0", "none", "null"),
    ];

    if let Ok(custom) = state.network_store.list().await {
        networks.extend(custom.into_iter().map(record_to_response));
    }

    Json(networks)
}

/// GET /networks/{id} — Inspect a network
pub async fn inspect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let builtin = match id.as_str() {
        "bridge" => Some(builtin_network("bridge", "bridge", "bridge")),
        "host" | "host0host0" => Some(builtin_network("host0host0", "host", "host")),
        "none" | "none0none0" => Some(builtin_network("none0none0", "none", "null")),
        _ => None,
    };

    if let Some(net) = builtin {
        return (axum::http::StatusCode::OK, Json(serde_json::to_value(net).unwrap_or_default()));
    }

    match state.network_store.get(&id).await {
        Ok(Some(rec)) => (axum::http::StatusCode::OK, Json(serde_json::to_value(record_to_response(rec)).unwrap_or_default())),
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
    let id = Uuid::new_v4().to_string();
    info!("Creating network '{}' with driver '{}'", body.name, body.driver.as_deref().unwrap_or("bridge"));

    let record = NetworkRecord {
        id: id.clone(),
        name: body.name,
        driver: body.driver.unwrap_or_else(|| "bridge".to_string()),
        labels: body.labels.unwrap_or_default(),
    };

    if let Err(e) = state.network_store.save(&record).await {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        );
    }

    (axum::http::StatusCode::CREATED, Json(serde_json::json!({ "Id": id })))
}

/// DELETE /networks/{id} — Remove a network
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if matches!(id.as_str(), "bridge" | "host" | "none") {
        return axum::http::StatusCode::FORBIDDEN;
    }

    match state.network_store.delete(&id).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(_) => axum::http::StatusCode::NOT_FOUND,
    }
}
