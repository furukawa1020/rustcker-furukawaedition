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
    if let Err(e) = state.container_store.delete(&id).await {
        error!("Failed to delete container from DB: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // 4. Cleanup Logs (Best effort)
    let log_path = std::path::Path::new("furukawa_logs").join(format!("{}.log", id));
    if log_path.exists() {
        if let Err(e) = tokio::fs::remove_file(&log_path).await {
            error!("Failed to remove log file for {}: {}", id, e);
            // We don't fail the request, just log error
        } else {
            info!("Removed log file for {}", id);
        }
    }

    info!(id = %id, "Container removed");
    
    StatusCode::NO_CONTENT
}
