use axum::{extract::{Path, State}, response::IntoResponse, http::StatusCode};
use tracing::{info, error};
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!(id = %id, "Received start container request");

    // 1. Load Container (Created state)
    let container = match state.container_store.get(&id).await {
        Ok(Some(c)) => c,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(e) => {
            error!("Failed to load container: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // 2. Start via Runtime
    let running_container = match container.start(state.runtime.as_ref()).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to start container: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
            // TODO: Map to 304 if already started, 400 if error
        }
    };

    // 3. Persist New State
    if let Err(e) = state.container_store.save_running(&running_container).await {
        error!("Failed to save running state: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    info!(id = %id, pid = %running_container.state().pid, "Container started successfully");
    
    StatusCode::NO_CONTENT
}
