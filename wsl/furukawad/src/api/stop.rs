use axum::{extract::{Path, State}, response::IntoResponse, http::StatusCode};
use tracing::{info, error};
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!(id = %id, "Received stop container request");

    // 1. Load Container (Running state)
    // We specifically need a running container to stop it.
    let container = match state.container_store.get_running(&id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            // It might exist but be stopped or created.
            // Ideally we check if it exists at all to return 304 or 404.
            // For now, 404 if not running is acceptable for prototype.
            // Strict Docker API returns 304 if already stopped.
            // We'd need a generic get_any_state to differentiate.
            // But let's stick to 404 for "Running container not found".
            return StatusCode::NOT_FOUND;
        }
        Err(e) => {
            error!("Failed to load container: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // 2. Stop via Runtime
    let stopped_container = match container.stop(state.runtime.as_ref()).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to stop container: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // 3. Persist New State
    if let Err(e) = state.container_store.save_stopped(&stopped_container).await {
        error!("Failed to save stopped state: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    info!(id = %id, exit_code = %stopped_container.state().exit_code, "Container stopped successfully");
    
    StatusCode::NO_CONTENT
}
