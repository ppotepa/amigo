use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSettingsChangedPayload {
    pub active_theme_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontSettingsChangedPayload {
    pub active_font_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInvalidatedPayload {
    pub project_cache_id: Option<String>,
    pub mod_id: Option<String>,
    pub scene_id: Option<String>,
    pub source_hash: Option<String>,
    pub cache_kind: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionClosedPayload {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRegistryChangedPayload {
    pub mod_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDescriptorChangedPayload {
    pub mod_id: String,
    pub asset_key: String,
    pub descriptor_relative_path: String,
    pub reason: String,
}
