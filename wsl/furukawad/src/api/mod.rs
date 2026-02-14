pub mod create;
pub mod middleware;


use axum::{routing::{get, post}, Router};

pub fn router() -> Router {
    Router::new()
        .route("/version", get(version_handler))
        .route("/containers/create", post(create::handle))
        .layer(axum::middleware::from_fn(middleware::trace_request))
}

async fn version_handler() -> &'static str {
    // This will eventually return strictly typed JSON.
    // For scaffolding verification, simple string is enough to prove flow.
    "{\"ApiVersion\":\"1.45\",\"Platform\":{\"Name\":\"furukawa-engine\"}}"
}
