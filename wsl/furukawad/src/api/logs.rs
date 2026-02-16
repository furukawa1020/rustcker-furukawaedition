use axum::{extract::{Path, State}, response::IntoResponse, http::StatusCode};
use tracing::error;
use crate::state::AppState;
use std::path::Path as StdPath;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // 1. Check if container exists
    if let Err(_) = state.container_store.get_status(&id).await {
        return StatusCode::NOT_FOUND.into_response();
    }

    // 2. Construct log path
    let log_path = StdPath::new("furukawa_logs").join(format!("{}.log", id));

    if !log_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    // 3. Read file
    match File::open(&log_path).await {
        Ok(mut file) => {
            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents).await {
                error!("Failed to read log file: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            (StatusCode::OK, contents).into_response()
        },
        Err(e) => {
            error!("Failed to open log file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
