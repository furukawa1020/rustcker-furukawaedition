pub mod create;
pub mod list;
pub mod start;
pub mod stop;
pub mod delete;
pub mod logs;
pub mod inspect;
pub mod version;
pub mod info;
pub mod images;
pub mod middleware;
pub mod networks;
pub mod build;
pub mod compose;

use axum::{routing::{get, post, delete as axum_delete}, Router};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        // System
        .route("/version", get(version::handle))
        .route("/info", get(info::handle))
        // Images
        .route("/images/json", get(images::list::handle))
        .route("/images/create", post(images::create::handle))
        // Containers
        .route("/containers/create", post(create::handle))
        .route("/containers/json", get(list::handle))
        .route("/containers/:id/start", post(start::handle))
        .route("/containers/:id/stop", post(stop::handle))
        .route("/containers/:id/logs", get(logs::handle))
        .route("/containers/:id/json", get(inspect::handle))
        .route("/containers/:id", axum_delete(delete::handle))
        // Networks
        .route("/networks", get(networks::list))
        .route("/networks/create", post(networks::create))
        .route("/networks/:id", get(networks::inspect))
        .route("/networks/:id", axum_delete(networks::delete))
        // Build
        .route("/build", post(build::handle))
        // Compose
        .route("/compose/up", post(compose::up))
        .route("/compose/down", post(compose::down))
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(axum::middleware::from_fn(middleware::trace_request))
        .with_state(state)
}
