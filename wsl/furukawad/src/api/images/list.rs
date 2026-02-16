use axum::response::IntoResponse;
use axum::Json;
use furukawa_infra_docker::v1_45::ImageSummary;

pub async fn handle() -> impl IntoResponse {
    // Current Limitation: We don't have a real image store query yet.
    // Return empty list to satisfy CLI, or maybe read form store?
    // For now, keep as empty.
    let images: Vec<ImageSummary> = vec![];
    Json(images).into_response()
}
