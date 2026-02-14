pub mod diagnostic;
pub mod telemetry;

// Universal Result type for the crate
pub type Result<T> = std::result::Result<T, diagnostic::Error>;
