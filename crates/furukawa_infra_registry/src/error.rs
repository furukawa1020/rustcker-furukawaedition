use thiserror::Error;
use furukawa_common::diagnostic::Diagnosable;

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
    fn code(&self) -> String {
        match self {
            Self::AuthenticationFailed(_) => "REG_AUTH_FAILED".to_string(),
            Self::ManifestNotFound(_) => "REG_MANIFEST_MISSING".to_string(),
            Self::BlobNotFound(_) => "REG_BLOB_MISSING".to_string(),
            Self::Network(_) => "REG_NETWORK_ERROR".to_string(),
            Self::Io(_) => "FS_IO_ERROR".to_string(),
            Self::InvalidDigest(_) => "REG_DIGEST_MISMATCH".to_string(),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::AuthenticationFailed(_) => Some("Check your internet connection or Docker Hub credentials.".to_string()),
            Self::ManifestNotFound(_) => Some("The image or tag might not exist. Check spelling.".to_string()),
            _ => None,
        }
    }
}
