use furukawa_infra_registry::RegistryClient;
use furukawa_infra_fs::store::image::ImageStore;
use furukawa_domain::image::store::ImageMetadataStore;

#[derive(Clone)]
pub struct AppState {
    pub container_store: Arc<dyn ContainerStore>,
    pub runtime: Arc<dyn ContainerRuntime>,
    pub registry: RegistryClient,
    pub image_store: Arc<ImageStore>,
    pub image_metadata_store: Arc<dyn ImageMetadataStore>,
}

impl AppState {
    pub fn new(
        store: SqliteStore, 
        runtime: ProcessRuntime,
        registry: RegistryClient,
        image_store: Arc<ImageStore>
    ) -> Self {
        let store = Arc::new(store);
        Self {
            container_store: store.clone(),
            runtime: Arc::new(runtime),
            registry,
            image_store,
            image_metadata_store: store,
        }
    }
}
