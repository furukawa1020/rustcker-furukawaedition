use axum::{extract::{Path, State}, response::IntoResponse, http::StatusCode};
use tracing::{info, error};
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // 1. Check existence and state
    let status = match state.container_store.get_status(&id).await {
        Ok(Some(s)) => s,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(e) => {
            error!("Failed to check container status: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // 2. Enforce "Stopped" state constraint (Strict Mode)
    // Docker Allows removing "created" containers too.
    if status == "running" {
        error!(id = %id, "Attempted to remove running container");
        return StatusCode::CONFLICT; 
    }

    // 3. Delete from DB
    // In a "10 year" impl, we would also remove log files, checking config for volumes, etc.
    if let Err(e) = state.container_store.delete(&id).await {
        error!("Failed to delete container: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    info!(id = %id, "Container removed");
    
    StatusCode::NO_CONTENT
}
