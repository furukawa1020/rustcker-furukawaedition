pub mod create;
pub mod list;
pub mod start;
pub mod stop;
pub mod delete;
pub mod logs;
pub mod inspect;
pub mod version;
pub mod middleware;

use axum::{routing::{get, post}, Router};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/version", get(version::handle))
        .route("/containers/create", post(create::handle))
        .route("/containers/json", get(list::handle))
        .route("/containers/:id/start", post(start::handle))
        .route("/containers/:id/stop", post(stop::handle))
        .route("/containers/:id/logs", get(logs::handle))
        .route("/containers/:id/json", get(inspect::handle))
        .route("/containers/:id", axum::routing::delete(delete::handle))
        .layer(axum::middleware::from_fn(middleware::trace_request))
        .with_state(state)
}

