use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use crate::state::AppState;
use furukawa_infra_docker::v1_45::ImageSummary;

pub async fn handle(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let images_metadata = match state.image_metadata_store.list().await {
        Ok(images) => images,
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list images: {}", e)).into_response(),
    };

    let mut summaries = Vec::new();
    for meta in images_metadata {
        summaries.push(ImageSummary {
            id: format!("sha256:{}", meta.id),
            parent_id: meta.parent_id.unwrap_or_default(), // Should maybe prefix sha256?
            repo_tags: Some(meta.repo_tags),
            repo_digests: None, // We don't track digests yet?
            created: meta.created,
            size: meta.size,
            shared_size: -1,
            virtual_size: meta.size,
            labels: None,
            containers: -1,
        });
    }

    Json(summaries).into_response()
}
