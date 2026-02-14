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
    
    let app = api::router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:2375").await.unwrap();
    info!("API Server listening on 127.0.0.1:2375");
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

mod api;
