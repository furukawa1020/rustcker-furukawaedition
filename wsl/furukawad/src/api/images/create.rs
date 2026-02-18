use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use crate::state::AppState;
use furukawa_infra_registry::manifest::ManifestV2;
use furukawa_domain::image::store::ImageMetadata;

#[derive(Deserialize)]
pub struct CreateImageParams {
    #[serde(rename = "fromImage")]
    from_image: String,
    tag: Option<String>,
}

pub async fn handle(
    State(state): State<AppState>,
    Query(params): Query<CreateImageParams>,
) -> impl IntoResponse {
    let repo = if params.from_image.contains('/') {
        params.from_image.clone()
    } else {
        format!("library/{}", params.from_image)
    };
    let tag = params.tag.as_deref().unwrap_or("latest");
    
    tracing::info!("Pulling image: {}:{}", repo, tag);
    
    // 1. Fetch Manifest
    let manifest_bytes = match state.registry.get_manifest(&repo, tag).await {
        Ok(bytes) => bytes,
        Err(e) => return (axum::http::StatusCode::BAD_REQUEST, format!("Failed to get manifest: {}", e)).into_response(),
    };
    
    let manifest: ManifestV2 = match serde_json::from_slice(&manifest_bytes) {
        Ok(m) => m,
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse manifest: {}", e)).into_response(),
    };
    
    // 2. Fetch Config
    let config_digest = &manifest.config.digest;
    tracing::info!("Fetching config: {}", config_digest);
    let config_bytes = match state.registry.get_blob(&repo, config_digest).await {
         Ok(bytes) => bytes,
         Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get config: {}", e)).into_response(),
    };
    
    let config_json: serde_json::Value = match serde_json::from_slice(&config_bytes) {
        Ok(v) => v,
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse config JSON: {}", e)).into_response(),
    };

    // Save config
    // Config Digest usually starts with "sha256:". ID is the hex part.
    let image_id = config_digest.strip_prefix("sha256:").unwrap_or(config_digest);
    if let Err(e) = state.image_store.save_config(image_id, config_json.clone()).await {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save config: {}", e)).into_response();
    }
    
    // 3. Fetch Layers
    let mut total_size = 0;
    for layer in &manifest.layers {
        tracing::info!("Fetching layer: {}", layer.digest);
        let layer_bytes = match state.registry.get_blob(&repo, &layer.digest).await {
             Ok(bytes) => bytes,
             Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get layer {}: {}", layer.digest, e)).into_response(),
        };
        
        if let Err(e) = state.image_store.save_layer(&layer.digest, layer_bytes).await {
             return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save layer {}: {}", layer.digest, e)).into_response();
        }
        total_size += layer.size;
    }
    
    // 4. Save Metadata
    // Extract 'created' from config if possible
    let created_str = config_json.get("created").and_then(|v| v.as_str()).unwrap_or("");
    let created_timestamp = match time::OffsetDateTime::parse(created_str, &time::format_description::well_known::Rfc3339) {
        Ok(t) => t.unix_timestamp(),
        Err(_) => time::OffsetDateTime::now_utc().unix_timestamp(),
    };
    
    let metadata = ImageMetadata {
        id: image_id.to_string(),
        repo_tags: vec![format!("{}:{}", repo, tag)], 
        parent_id: config_json.get("parent").and_then(|v| v.as_str()).map(|s| s.to_string()),
        created: created_timestamp,
        size: total_size,
    };
    
    if let Err(e) = state.image_metadata_store.save(&metadata).await {
         return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save metadata: {}", e)).into_response();
    }
    
    tracing::info!("Image pulled successfully: {}", image_id);

    "Pull complete".into_response()
}
