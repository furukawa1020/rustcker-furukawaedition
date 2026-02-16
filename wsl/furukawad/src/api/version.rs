use axum::{response::IntoResponse, Json};
use furukawa_infra_docker::v1_45::{Version, Platform, Component};

pub async fn handle() -> impl IntoResponse {
    let version = Version {
        platform: Platform {
            name: "Furukawa Engine".to_string(),
        },
        components: Some(vec![
            Component {
                name: "Engine".to_string(),
                version: "0.1.0".to_string(),
                details: None,
            }
        ]),
        version: "0.1.0".to_string(),
        api_version: "1.45".to_string(),
        min_a_p_i_version: "1.45".to_string(),
        git_commit: "HEAD".to_string(),
        go_version: "rust".to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        kernel_version: "unknown".to_string(),
        experimental: false,
        build_time: None,
    };

    Json(version)
}
