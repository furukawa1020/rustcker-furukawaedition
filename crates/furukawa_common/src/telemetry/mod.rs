use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing(service_name: &str) -> anyhow::Result<()> {
    // Basic setup for now. In the future, this will configure OpenTelemetry.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to init tracing: {}", e))?;

    tracing::info!("Tracing initialized for service: {}", service_name);
    Ok(())
}
