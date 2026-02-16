use furukawa_infra_registry::RegistryClient;
use furukawa_infra_fs::store::image::ImageStore;

#[derive(Clone)]
pub struct AppState {
    pub container_store: Arc<dyn ContainerStore>,
    pub runtime: Arc<dyn ContainerRuntime>,
    pub registry: RegistryClient,
    pub image_store: Arc<ImageStore>,
}

impl AppState {
    pub fn new(
        store: SqliteStore, 
        runtime: ProcessRuntime,
        registry: RegistryClient,
        image_store: Arc<ImageStore>
    ) -> Self {
        Self {
            container_store: Arc::new(store),
            runtime: Arc::new(runtime),
            registry,
            image_store,
        }
    }
}
