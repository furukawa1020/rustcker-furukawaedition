use furukawa_common::telemetry;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init_tracing("furukawad")?;
    
    info!("Starting HATAKE Desktop Engine (furukawad) - strictly compliant mode");
    
    // Resolve data directory from env var (set by Tauri sidecar) or fallback to cwd
    let data_dir = std::env::var("FURUKAWA_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    // Ensure the data directory exists
    std::fs::create_dir_all(&data_dir)?;

    // ── Phase 6A: WSL Distro Auto-Setup ────────────────────────────────────
    let distro_name = std::env::var("FURUKAWA_DISTRO")
        .unwrap_or_else(|_| "furukawa-alpine".to_string());
    let wsl_manager = furukawa_infra_wsl::WslManager::new(
        distro_name.clone(),
        data_dir.join("wsl"),
    );
    
    if let Err(e) = wsl_manager.ensure_distro().await {
        tracing::warn!("WSL distro setup failed (non-fatal): {}. Falling back to Ubuntu.", e);
    }

    // ── Database ─────────────────────────────────────────────────────────────
    let db_path = data_dir.join("furukawa.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy().replace('\\', "/"));
    
    info!("Using database at: {}", db_url);
    let sqlite_store = furukawa_infra_db::SqliteStore::new(&db_url).await?;
    let store = std::sync::Arc::new(sqlite_store);
    
    // ── Registry + Image Store ────────────────────────────────────────────────
    let registry = furukawa_infra_registry::RegistryClient::new();
    let furukawa_data = data_dir.join("furukawa_data");
    let image_store = std::sync::Arc::new(furukawa_infra_fs::store::image::ImageStore::new(
        furukawa_data.clone()
    ));
    image_store.ensure_dirs().await?;

    // ── Runtime (WSL2) ────────────────────────────────────────────────────────
    let runtime = furukawa_infra_runtime::WslRuntime {
        image_store: image_store.clone(),
        metadata_store: store.clone(),
        containers_root: furukawa_data.join("containers"),
        distro: distro_name,
    };

    // ── App State ───────────────────────────────────────────────────────────
    let state = state::AppState {
        container_store: store.clone(),
        runtime: std::sync::Arc::new(runtime),
        registry,
        image_store,
        image_metadata_store: store.clone(),
        network_store: store,
    };

    // ── Start API Server ─────────────────────────────────────────────────────
    let app = api::router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:2375").await?;
    info!("API Server listening on 127.0.0.1:2375");
    axum::serve(listener, app).await?;
    
    Ok(())
}

mod api;
mod state;
