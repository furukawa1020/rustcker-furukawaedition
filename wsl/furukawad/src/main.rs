use furukawa_common::telemetry;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init_tracing("rustkerd")?;
    
    info!("Starting Rustker Desktop Engine (rustkerd) - strictly compliant mode");
    
    // Resolve data directory from env var (set by Tauri sidecar) or fallback to cwd
    let data_dir = std::env::var("RUSTKER_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    // Ensure the data directory exists
    std::fs::create_dir_all(&data_dir)?;

    // ── Phase 6A: WSL Distro Auto-Setup ────────────────────────────────────
    let distro_name = std::env::var("RUSTKER_DISTRO")
        .unwrap_or_else(|_| "rustker-alpine".to_string());
    
    let skip_wsl_setup = std::env::var("RUSTKER_SKIP_WSL_SETUP").is_ok();
    if !skip_wsl_setup {
        let wsl_manager = furukawa_infra_wsl::WslManager::new(
            distro_name.clone(),
            data_dir.join("wsl"),
        );
        // Run distro setup with a 30-second timeout so slow WSL operations don't block startup
        let setup_result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            wsl_manager.ensure_distro(),
        ).await;
        match setup_result {
            Ok(Ok(())) => info!("WSL distro setup complete."),
            Ok(Err(e)) => tracing::warn!("WSL distro setup failed (non-fatal): {}", e),
            Err(_) => tracing::warn!("WSL distro setup timed out (non-fatal). Engine will continue."),
        }
    } else {
        info!("RUSTKER_SKIP_WSL_SETUP set  Eskipping WSL distro auto-setup.");
    }

    // ── Database ─────────────────────────────────────────────────────────────
    let db_path = data_dir.join("rustker.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy().replace('\\', "/"));
    
    info!("Using database at: {}", db_url);
    let sqlite_store = furukawa_infra_db::SqliteStore::new(&db_url).await?;
    let store = std::sync::Arc::new(sqlite_store);
    
    // ── Registry + Image Store ────────────────────────────────────────────────
    let registry = furukawa_infra_registry::RegistryClient::new();
    let rustker_data = data_dir.join("rustker_data");
    let image_store = std::sync::Arc::new(furukawa_infra_fs::store::image::ImageStore::new(
        rustker_data.clone()
    ));
    image_store.ensure_dirs().await?;

    // ── Runtime (WSL2) ────────────────────────────────────────────────────────
    let runtime = furukawa_infra_runtime::WslRuntime {
        image_store: image_store.clone(),
        metadata_store: store.clone(),
        containers_root: rustker_data.join("containers"),
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
