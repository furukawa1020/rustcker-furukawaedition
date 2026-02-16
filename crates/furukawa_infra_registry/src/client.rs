use crate::auth::Authenticator;
use crate::error::RegistryError;
use reqwest::{Client, StatusCode, header};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use bytes::Bytes;
use futures_util::StreamExt;

const DEFAULT_REGISTRY: &str = "https://registry-1.docker.io";

#[derive(Clone)]
pub struct RegistryClient {
    client: Client,
    authenticator: Arc<Mutex<Authenticator>>,
    registry_url: String,
}

impl RegistryClient {
    pub fn new() -> Self {
        let client = Client::new();
        Self {
            client: client.clone(),
            authenticator: Arc::new(Mutex::new(Authenticator::new(client))),
            registry_url: DEFAULT_REGISTRY.to_string(),
        }
    }

    async fn authenticate_if_needed(&self, repo: &str, scope: &str) -> Result<String, RegistryError> {
        // Optimistically try without auth first? Actually Docker Hub requires anon token for public images.
        // We simulate the flow:
        // 1. Request -> 401 Www-Authenticate: Bearer realm="...",service="...",scope="..."
        // 2. Auth -> Token
        
        // For simplicity, we hardcode logic for Docker Hub for now.
        // Real implementation should parse Www-Authenticate header.
        let realm = "https://auth.docker.io/token";
        let service = "registry.docker.io";
        let full_scope = format!("repository:{}:pull", repo);

        let mut auth = self.authenticator.lock().await;
        auth.get_token(realm, service, &full_scope).await
    }

    pub async fn get_manifest(&self, repo: &str, reference: &str) -> Result<Bytes, RegistryError> {
        let url = format!("{}/v2/{}/manifests/{}", self.registry_url, repo, reference);
        let token = self.authenticate_if_needed(repo, "pull").await?;

        let resp = self.client.get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.docker.distribution.manifest.v2+json")
            .header(header::ACCEPT, "application/vnd.docker.distribution.manifest.list.v2+json")
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.bytes().await?)
        } else if resp.status() == StatusCode::NOT_FOUND {
            Err(RegistryError::ManifestNotFound(reference.to_string()))
        } else {
            Err(RegistryError::Network(resp.error_for_status().unwrap_err())) // Should handle better
        }
    }

    pub async fn get_blob(&self, repo: &str, digest: &str) -> Result<Bytes, RegistryError> {
        let url = format!("{}/v2/{}/blobs/{}", self.registry_url, repo, digest);
        let token = self.authenticate_if_needed(repo, "pull").await?;

        let resp = self.client.get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        if resp.status().is_success() {
             Ok(resp.bytes().await?)
        } else if resp.status() == StatusCode::NOT_FOUND {
            Err(RegistryError::BlobNotFound(digest.to_string()))
        } else {
             Err(RegistryError::Network(resp.error_for_status().unwrap_err()))
        }
    }
}
