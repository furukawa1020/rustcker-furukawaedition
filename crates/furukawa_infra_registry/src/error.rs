use thiserror::Error;
use furukawa_common::diagnostic::{Diagnosable, DiagnosticCode, Severity};

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Failed to authenticate with registry: {0}")]
    AuthenticationFailed(String),
    #[error("Manifest not found: {0}")]
    ManifestNotFound(String),
    #[error("Blob not found: {0}")]
    BlobNotFound(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid digest: {0}")]
    InvalidDigest(String),
}

impl Diagnosable for RegistryError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::AuthenticationFailed(_) => DiagnosticCode("REG_AUTH_FAILED"),
            Self::ManifestNotFound(_) => DiagnosticCode("REG_MANIFEST_MISSING"),
            Self::BlobNotFound(_) => DiagnosticCode("REG_BLOB_MISSING"),
            Self::Network(_) => DiagnosticCode("REG_NETWORK_ERROR"),
            Self::Io(_) => DiagnosticCode("FS_IO_ERROR"),
            Self::InvalidDigest(_) => DiagnosticCode("REG_DIGEST_MISMATCH"),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::AuthenticationFailed(_) => Some("Check your internet connection or Docker Hub credentials.".to_string()),
            Self::ManifestNotFound(_) => Some("The image or tag might not exist. Check spelling.".to_string()),
            _ => None,
        }
    }
}
