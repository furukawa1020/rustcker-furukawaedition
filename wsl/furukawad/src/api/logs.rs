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
            let mut bytes = Vec::new();
            if let Err(e) = file.read_to_end(&mut bytes).await {
                error!("Failed to read log file: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }

            let contents = if bytes.starts_with(&[0xff, 0xfe]) {
                // UTF-16 LE
                let u16_data: Vec<u16> = bytes[2..]
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                String::from_utf16_lossy(&u16_data)
            } else {
                String::from_utf8_lossy(&bytes).into_owned()
            };

            (StatusCode::OK, contents).into_response()
        },
        Err(e) => {
            error!("Failed to open log file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
