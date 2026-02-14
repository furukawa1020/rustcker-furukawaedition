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
    
    Ok(())
}
