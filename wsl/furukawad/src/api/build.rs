use axum::{extract::{Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct BuildQuery {
    /// Tag to apply to the built image (e.g. "myapp:latest")
    pub t: Option<String>,
    pub dockerfile: Option<String>,
}

/// POST /build
///
/// Accepts a tar archive of the build context as the body.
/// Extracts it to a temp directory and runs the Dockerfile build.
pub async fn handle(
    State(state): State<AppState>,
    Query(params): Query<BuildQuery>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let tag = params.t.unwrap_or_else(|| "furukawa-built:latest".to_string());
    let dockerfile_name = params.dockerfile.unwrap_or_else(|| "Dockerfile".to_string());
    
    info!("POST /build tag={} dockerfile={}", tag, dockerfile_name);

    // Extract the build context tar to a temp dir
    let build_context_dir = std::path::Path::new("furukawa_build_ctx")
        .join(uuid::Uuid::new_v4().to_string());
    
    if let Err(e) = std::fs::create_dir_all(&build_context_dir) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to create build context dir: {}", e)})),
        );
    }

    // Untar the body
    if !body.is_empty() {
        let cursor = std::io::Cursor::new(body.as_ref());
        let gz = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(gz);
        if let Err(e) = archive.unpack(&build_context_dir) {
            // Try uncompressed tar
            let cursor = std::io::Cursor::new(body.as_ref());
            let mut archive = tar::Archive::new(cursor);
            if let Err(e2) = archive.unpack(&build_context_dir) {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": format!("Failed to unpack build context: {} / {}", e, e2)})),
                );
            }
        }
    }

    // Read Dockerfile
    let dockerfile_path = build_context_dir.join(&dockerfile_name);
    let dockerfile_content = match std::fs::read_to_string(&dockerfile_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Cannot read {}: {}", dockerfile_name, e)})),
            );
        }
    };

    // Get WSL distro from state (via runtime downcast approximation)
    let distro = "rustker-alpine"; // fallback; ideally read from config
    let output_dir = std::path::Path::new("furukawa_build_out");

    let build_ctx = match furukawa_build::BuildContext::new(
        build_context_dir.clone(),
        &dockerfile_content,
        &tag,
        distro,
    ) {
        Ok(ctx) => ctx,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Dockerfile parse error: {}", e)})),
            );
        }
    };

    if std::fs::create_dir_all(output_dir).is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create build output dir"})),
        );
    }

    // Run the build (stream would be nicer, but sync JSON response for now)
    match furukawa_build::run_build(&build_ctx, output_dir).await {
        Ok(layer_path) => {
            info!("Build succeeded, layer at {:?}", layer_path);
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({
                    "stream": format!("Successfully built {}\n", tag),
                    "tag": tag,
                })),
            )
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}
