

/// A trait for errors that provide a diagnostic code and a suggestion for resolution.
pub trait Diagnosable: std::error::Error {
    /// A unique machine-readable code (e.g., "FS_WRITE_FAILED").
    fn code(&self) -> String;

    /// A human-readable suggestion for how to fix the error.
    fn suggestion(&self) -> Option<String>;
}

#[derive(Debug, thiserror::Error)]
#[error("{message} (Code: {code})")]
pub struct Error {
    message: String,
    code: String,
    suggestion: Option<String>,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn new<E>(error: E) -> Self
    where
        E: Diagnosable + Send + Sync + 'static,
    {
        Self {
            message: error.to_string(),
            code: error.code(),
            suggestion: error.suggestion(),
            source: Some(Box::new(error)),
        }
    }
}
