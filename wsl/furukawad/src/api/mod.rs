pub mod create;
pub mod list;
pub mod start;
pub mod stop;
pub mod middleware;

use axum::{routing::{get, post}, Router};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/version", get(version_handler))
        .route("/containers/create", post(create::handle))
        .route("/containers/json", get(list::handle))
        .route("/containers/:id/start", post(start::handle))
        .route("/containers/:id/stop", post(stop::handle))
        .layer(axum::middleware::from_fn(middleware::trace_request))
        .with_state(state)
}

async fn version_handler() -> &'static str {
    // This will eventually return strictly typed JSON.
    // For scaffolding verification, simple string is enough to prove flow.
    "{\"ApiVersion\":\"1.45\",\"Platform\":{\"Name\":\"furukawa-engine\"}}"
}
