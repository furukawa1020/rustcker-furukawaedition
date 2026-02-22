//! Docker Compose v3 YAML parser and multi-container runner for HATAKE Desktop.
//!
//! Supports: services (image, ports, volumes, environment, depends_on, command)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

/// Represents a parsed `compose.yml` file.
#[derive(Debug, Deserialize)]
pub struct ComposeFile {
    #[serde(default)]
    pub version: Option<String>,
    pub services: HashMap<String, ServiceConfig>,
    #[serde(default)]
    pub networks: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub volumes: HashMap<String, serde_yaml::Value>,
}

/// Configuration for a single Compose service.
#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub image: Option<String>,
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub command: CommandField,
    #[serde(default)]
    pub ports: Vec<String>,
    #[serde(default)]
    pub volumes: Vec<String>,
    #[serde(default)]
    pub environment: EnvField,
    #[serde(default)]
    pub depends_on: DependsOnField,
    pub network_mode: Option<String>,
    pub restart: Option<String>,
}

/// build: context or object form
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum BuildConfig {
    Context(String),
    Object { context: String, dockerfile: Option<String> },
}

/// command field can be string or sequence
#[derive(Debug, Default, Deserialize, Clone)]
#[serde(untagged)]
pub enum CommandField {
    #[default]
    None,
    String(String),
    List(Vec<String>),
}

impl CommandField {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::None => vec![],
            Self::String(s) => vec!["sh".to_string(), "-c".to_string(), s.clone()],
            Self::List(v) => v.clone(),
        }
    }
}

/// environment field can be a map or a list of KEY=VALUE strings
#[derive(Debug, Default, Deserialize, Clone)]
#[serde(untagged)]
pub enum EnvField {
    #[default]
    None,
    Map(HashMap<String, String>),
    List(Vec<String>),
}

impl EnvField {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::None => vec![],
            Self::Map(m) => m.iter().map(|(k, v)| format!("{}={}", k, v)).collect(),
            Self::List(v) => v.clone(),
        }
    }
}

/// depends_on can be a list of service names or a map
#[derive(Debug, Default, Deserialize, Clone)]
#[serde(untagged)]
pub enum DependsOnField {
    #[default]
    None,
    List(Vec<String>),
    Map(HashMap<String, serde_yaml::Value>),
}

impl DependsOnField {
    pub fn service_names(&self) -> Vec<String> {
        match self {
            Self::None => vec![],
            Self::List(v) => v.clone(),
            Self::Map(m) => m.keys().cloned().collect(),
        }
    }
}

/// Parse a compose.yml string into a `ComposeFile`.
pub fn parse_compose(content: &str) -> Result<ComposeFile> {
    serde_yaml::from_str(content).context("Failed to parse compose.yml")
}

/// Topologically sort services by `depends_on` so they start in correct order.
pub fn sorted_services(compose: &ComposeFile) -> Result<Vec<String>> {
    let mut order = Vec::new();
    let mut visited = std::collections::HashSet::new();

    fn visit(
        name: &str,
        services: &HashMap<String, ServiceConfig>,
        visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if visited.contains(name) {
            return Ok(());
        }
        visited.insert(name.to_string());
        let service = services.get(name)
            .context(format!("Service '{}' referenced in depends_on but not defined", name))?;
        for dep in service.depends_on.service_names() {
            visit(&dep, services, visited, order)?;
        }
        order.push(name.to_string());
        Ok(())
    }

    for name in compose.services.keys() {
        visit(name, &compose.services, &mut visited, &mut order)?;
    }

    Ok(order)
}

/// Information about a started Compose service.
#[derive(Debug, Serialize)]
pub struct StartedService {
    pub service_name: String,
    pub container_id: String,
}

/// Request to bring up Compose services by calling the furukawad API.
pub async fn compose_up(
    compose: &ComposeFile,
    api_base: &str,
    project_name: &str,
) -> Result<Vec<StartedService>> {
    let client = reqwest::Client::new();
    let order = sorted_services(compose)?;
    let mut started = Vec::new();

    for service_name in &order {
        let service = &compose.services[service_name];
        let image = service.image.as_deref().unwrap_or("alpine:latest");

        info!("[COMPOSE] Starting service '{}' from image '{}'", service_name, image);

        // Build port bindings
        let mut port_bindings = serde_json::Map::<String, Value>::new();
        for port_spec in &service.ports {
            // "8080:80" or "80"
            let (host_port, container_port) = if let Some((h, c)) = port_spec.split_once(':') {
                (h, c)
            } else {
                (port_spec.as_str(), port_spec.as_str())
            };
            port_bindings.insert(
                format!("{}/tcp", container_port),
                serde_json::json!([{"HostPort": host_port}]),
            );
        }

        // Build volume binds
        let binds: Vec<String> = service.volumes.clone();

        let container_name = format!("{}-{}", project_name, service_name);

        // POST /containers/create
        let create_body = serde_json::json!({
            "Image": image,
            "Cmd": service.command.to_vec(),
            "Env": service.environment.to_vec(),
            "HostConfig": {
                "PortBindings": port_bindings,
                "Binds": binds,
                "NetworkMode": service.network_mode.as_deref().unwrap_or("bridge"),
            }
        });

        let create_resp = client.post(&format!("{}/containers/create?name={}", api_base, container_name))
            .json(&create_body)
            .send()
            .await
            .context("Failed to create container")?;

        let container_id = create_resp.json::<serde_json::Value>().await?
            .get("Id").and_then(|v: &serde_json::Value| v.as_str())
            .context("Missing container ID in response")?
            .to_string();

        // POST /containers/{id}/start
        client.post(&format!("{}/containers/{}/start", api_base, container_id))
            .send()
            .await
            .context("Failed to start container")?;

        info!("[COMPOSE] Service '{}' started as container {}", service_name, container_id);
        started.push(StartedService { service_name: service_name.clone(), container_id });
    }

    Ok(started)
}

/// Bring down all containers for a given project (identified by labels or naming convention).
pub async fn compose_down(
    compose: &ComposeFile,
    api_base: &str,
    project_name: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let order = sorted_services(compose)?;

    // Stop in reverse order
    for service_name in order.iter().rev() {
        let container_name = format!("{}-{}", project_name, service_name);
        info!("[COMPOSE] Stopping service '{}'", service_name);

        // GET /containers/json to find the container ID by name
        let list_resp = client.get(&format!("{}/containers/json?all=1", api_base))
            .send()
            .await?
            .json::<Vec<serde_json::Value>>()
            .await?;

        for c in &list_resp {
            let names = c.get("Names").and_then(|n: &serde_json::Value| n.as_array());
            let id = c.get("Id").and_then(|v: &serde_json::Value| v.as_str()).unwrap_or("");
            if let Some(names) = names {
                if names.iter().any(|n| n.as_str().unwrap_or("").contains(&container_name)) {
                    let _ = client.post(&format!("{}/containers/{}/stop", api_base, id)).send().await;
                    let _ = client.delete(&format!("{}/containers/{}", api_base, id)).send().await;
                }
            }
        }
    }

    Ok(())
}
