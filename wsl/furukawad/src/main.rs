use furukawa_common::telemetry;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init_tracing("furukawad")?;
    
    info!("Starting HATAKE Desktop Engine (furukawad) - strictly compliant mode");
    
    // In the future:
    // 1. Load config
    // 2. Init FSMs
    // 3. Start API Server
    
    // Check minimal dependencies
    let _ = furukawa_domain::container::Created;
    
    // Initialize Persistence
    // In production, this path comes from config
    let db_url = "sqlite://furukawa.db?mode=rwc"; 
    let sqlite_store = furukawa_infra_db::SqliteStore::new(db_url).await.unwrap();
    let store = std::sync::Arc::new(sqlite_store);
    
    let registry = furukawa_infra_registry::RegistryClient::new();
    let image_store = std::sync::Arc::new(furukawa_infra_fs::store::image::ImageStore::new(
        std::path::PathBuf::from("furukawa_data")
    ));
    image_store.ensure_dirs().await.unwrap();

    let runtime = furukawa_infra_runtime::WslRuntime {
        image_store: image_store.clone(),
        metadata_store: store.clone(),
        containers_root: std::path::PathBuf::from("furukawa_data/containers"),
        distro: "Ubuntu".to_string(), // Default for Phase 5 prototype
    };

    let state = state::AppState {
        container_store: store.clone(),
        runtime: std::sync::Arc::new(runtime),
        registry,
        image_store,
        image_metadata_store: store,
    };

    let app = api::router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:2375").await.unwrap();
    info!("API Server listening on 127.0.0.1:2375");
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

mod api;
mod state;
