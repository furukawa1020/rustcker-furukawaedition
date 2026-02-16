use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestV2 {
    pub schema_version: u32,
    pub media_type: String,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub media_type: String,
    pub size: i64,
    pub digest: String,
    pub urls: Option<Vec<String>>,
}
