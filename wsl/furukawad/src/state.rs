use furukawa_infra_registry::RegistryClient;
use furukawa_infra_fs::store::image::ImageStore;
use furukawa_domain::image::store::ImageMetadataStore;
use furukawa_domain::container::store::ContainerStore;
use furukawa_domain::container::runtime::ContainerRuntime;
use furukawa_domain::network::NetworkStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub container_store: Arc<dyn ContainerStore>,
    pub runtime: Arc<dyn ContainerRuntime>,
    pub registry: RegistryClient,
    pub image_store: Arc<ImageStore>,
    pub image_metadata_store: Arc<dyn ImageMetadataStore>,
    pub network_store: Arc<dyn NetworkStore>,
}
