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
    let store = furukawa_infra_db::SqliteStore::new(db_url).await.unwrap();
    let runtime = furukawa_infra_runtime::ProcessRuntime::default();
    let state = state::AppState::new(store, runtime);

    let app = api::router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:2375").await.unwrap();
    info!("API Server listening on 127.0.0.1:2375");
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

mod api;
mod state;
