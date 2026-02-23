use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use crate::state::AppState;

fn volumes_root() -> PathBuf {
    let data = std::env::var("RUSTKER_DATA_DIR").unwrap_or_else(|_| ".".into());
    PathBuf::from(data).join("rustker_data").join("volumes")
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub scope: String,
    pub labels: std::collections::HashMap<String, String>,
}

/// GET /volumes
pub async fn list(State(_state): State<AppState>) -> impl IntoResponse {
    let root = volumes_root();
    let _ = fs::create_dir_all(&root).await;

    let mut volumes = Vec::new();
    if let Ok(mut rd) = fs::read_dir(&root).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            let mp = entry.path().to_string_lossy().to_string();
            volumes.push(VolumeInfo {
                name,
                driver: "local".into(),
                mountpoint: mp,
                scope: "local".into(),
                labels: Default::default(),
            });
        }
    }

    // Docker API wraps volumes in { "Volumes": [...], "Warnings": null }
    Json(serde_json::json!({
        "Volumes": volumes,
        "Warnings": null
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateVolumeRequest {
    pub name: String,
    pub driver: Option<String>,
}

/// POST /volumes/create
pub async fn create(
    State(_state): State<AppState>,
    Json(body): Json<CreateVolumeRequest>,
) -> impl IntoResponse {
    let root = volumes_root();
    let vol_path = root.join(&body.name);

    if let Err(e) = fs::create_dir_all(&vol_path).await {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message": e.to_string()})),
        );
    }

    info!("Created volume '{}' at {:?}", body.name, vol_path);

    let mp = vol_path.to_string_lossy().to_string();
    (
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({
            "Name": body.name,
            "Driver": body.driver.unwrap_or_else(|| "local".into()),
            "Mountpoint": mp,
            "Scope": "local",
            "Labels": {}
        })),
    )
}

/// DELETE /volumes/{name}
pub async fn delete(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let root = volumes_root();
    let vol_path = root.join(&name);

    if vol_path.exists() {
        if let Err(e) = fs::remove_dir_all(&vol_path).await {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"message": e.to_string()})),
            );
        }
        info!("Deleted volume '{}'", name);
        (axum::http::StatusCode::NO_CONTENT, Json(serde_json::json!({})))
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": format!("volume '{}' not found", name)})),
        )
    }
}

/// DELETE /volumes/prune  Eremove all unused volumes
pub async fn prune(State(_state): State<AppState>) -> impl IntoResponse {
    let root = volumes_root();
    let mut pruned = Vec::new();
    if let Ok(mut rd) = fs::read_dir(&root).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if fs::remove_dir_all(entry.path()).await.is_ok() {
                pruned.push(name);
            }
        }
    }
    Json(serde_json::json!({
        "VolumesDeleted": pruned,
        "SpaceReclaimed": 0
    }))
}
