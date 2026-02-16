use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use crate::error::RegistryError;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: String,
    // access_token: Option<String>, // Some registries use this
    // expires_in: Option<u64>,
}

pub struct Authenticator {
    client: Client,
    tokens: HashMap<String, String>, // Scope -> Token
}

impl Authenticator {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            tokens: HashMap::new(),
        }
    }

    pub async fn get_token(&mut self, realm: &str, service: &str, scope: &str) -> Result<String, RegistryError> {
        // Simple caching strategy: if we recently fetched a token for this scope, return it.
        // TODO: Implement proper expiration handling.
        if let Some(token) = self.tokens.get(scope) {
            return Ok(token.clone());
        }

        let url = format!("{}?service={}&scope={}", realm, service, scope);
        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            return Err(RegistryError::AuthenticationFailed(format!("Status: {}", resp.status())));
        }

        let token_resp: TokenResponse = resp.json().await?;
        let token = token_resp.token; // .or(token_resp.access_token).ok_or(...)

        self.tokens.insert(scope.to_string(), token.clone());
        Ok(token)
    }
}
