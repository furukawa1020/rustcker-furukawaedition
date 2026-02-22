use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::state::AppState;

const API_BASE: &str = "http://127.0.0.1:2375";

#[derive(Deserialize)]
pub struct ComposeUpRequest {
    /// The raw content of docker-compose.yml
    pub compose_yaml: String,
    /// Project name (prefix for container names)
    pub project_name: Option<String>,
}

#[derive(Serialize)]
pub struct ComposeUpResponse {
    pub started: Vec<furukawa_compose::StartedService>,
}

/// POST /compose/up
/// Body: { compose_yaml: "...", project_name: "myapp" }
pub async fn up(
    State(_state): State<AppState>,
    Json(body): Json<ComposeUpRequest>,
) -> impl IntoResponse {
    let project_name = body.project_name.unwrap_or_else(|| "furukawa".to_string());
    info!("POST /compose/up project={}", project_name);

    let compose = match furukawa_compose::parse_compose(&body.compose_yaml) {
        Ok(c) => c,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("compose.yml parse error: {}", e)})),
            );
        }
    };

    match furukawa_compose::compose_up(&compose, API_BASE, &project_name).await {
        Ok(started) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({ "started": started })),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
pub struct ComposeDownRequest {
    pub compose_yaml: String,
    pub project_name: Option<String>,
}

/// POST /compose/down
pub async fn down(
    State(_state): State<AppState>,
    Json(body): Json<ComposeDownRequest>,
) -> impl IntoResponse {
    let project_name = body.project_name.unwrap_or_else(|| "furukawa".to_string());
    info!("POST /compose/down project={}", project_name);

    let compose = match furukawa_compose::parse_compose(&body.compose_yaml) {
        Ok(c) => c,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("compose.yml parse error: {}", e)})),
            );
        }
    };

    match furukawa_compose::compose_down(&compose, API_BASE, &project_name).await {
        Ok(_) => (axum::http::StatusCode::OK, Json(serde_json::json!({"status": "down"}))),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}
