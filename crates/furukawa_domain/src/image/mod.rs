use furukawa_common::diagnostic::{Diagnosable, Error};
use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DigestError {
    #[error("Invalid digest format: {0}")]
    InvalidFormat(String),
}

impl Diagnosable for DigestError {
    fn code(&self) -> String {
        "IMAGE_DIGEST_INVALID".to_string()
    }
    fn suggestion(&self) -> Option<String> {
        Some("Digest must be sha256:<hex>".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest(String);

impl Digest {
    pub fn new(s: impl Into<String>) -> Result<Self, Error> {
        let s = s.into();
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"^sha256:[a-f0-9]{64}$").unwrap());

        if re.is_match(&s) {
            Ok(Self(s))
        } else {
            Err(Error::new(DigestError::InvalidFormat(s)))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
