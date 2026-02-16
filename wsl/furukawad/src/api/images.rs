use axum::response::IntoResponse;
use axum::Json;
use furukawa_infra_docker::v1_45::ImageSummary;

pub async fn handle() -> impl IntoResponse {
    // Current Limitation: We don't have a real image store.
    // Return empty list to satisfy CLI.
    let images: Vec<ImageSummary> = vec![];
    Json(images).into_response()
}
