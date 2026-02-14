use furukawa_common::telemetry;

fn main() -> anyhow::Result<()> {
    telemetry::init_tracing("furukawa_service")?;
    tracing::info!("Starting HATAKE Desktop Windows Service");
    
    // Service logic placeholder
    
    Ok(())
}
