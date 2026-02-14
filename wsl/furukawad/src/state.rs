use furukawa_domain::container::store::ContainerStore;
use furukawa_infra_db::SqliteStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub container_store: Arc<dyn ContainerStore>,
}

impl AppState {
    pub fn new(store: SqliteStore) -> Self {
        Self {
            container_store: Arc::new(store),
        }
    }
}
