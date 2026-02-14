use furukawa_domain::container::store::ContainerStore;
use furukawa_domain::container::runtime::ContainerRuntime;
use furukawa_infra_db::SqliteStore;
use furukawa_infra_runtime::ProcessRuntime;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub container_store: Arc<dyn ContainerStore>,
    pub runtime: Arc<dyn ContainerRuntime>,
}

impl AppState {
    pub fn new(store: SqliteStore, runtime: ProcessRuntime) -> Self {
        Self {
            container_store: Arc::new(store),
            runtime: Arc::new(runtime),
        }
    }
}
