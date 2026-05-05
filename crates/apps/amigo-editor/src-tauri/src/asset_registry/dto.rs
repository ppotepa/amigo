use serde::{Deserialize, Serialize};

use crate::dto::EditorDiagnosticDto;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRegistryDto {
    pub session_id: String,
    pub mod_id: String,
    pub root_path: String,
    pub managed_assets: Vec<ManagedAssetDto>,
    pub raw_files: Vec<RawAssetFileDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedAssetDto {
    pub asset_id: String,
    pub kind: String,
    pub label: String,
    pub asset_key: String,
    pub parent_key: Option<String>,
    pub references: Vec<String>,
    pub used_by: Vec<String>,
    pub domain: AssetDomainDto,
    pub role: AssetRoleDto,
    pub descriptor_path: String,
    pub descriptor_relative_path: String,
    pub source_files: Vec<AssetSourceRefDto>,
    pub status: AssetStatusDto,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSourceRefDto {
    pub path: String,
    pub relative_path: String,
    pub exists: bool,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawAssetFileDto {
    pub path: String,
    pub relative_path: String,
    pub media_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub referenced_by: Vec<String>,
    pub orphan: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetStatusDto {
    Valid,
    Warning,
    Error,
    MissingSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum AssetDomainDto {
    Spritesheet,
    Tilemap,
    Audio,
    Font,
    Scene,
    Script,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum AssetRoleDto {
    Family,
    Subasset,
    Reference,
    File,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssetImportOptionsDto {
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub columns: Option<u32>,
    pub rows: Option<u32>,
    pub tile_count: Option<u32>,
    pub margin_x: Option<u32>,
    pub margin_y: Option<u32>,
    pub spacing_x: Option<u32>,
    pub spacing_y: Option<u32>,
    pub fps: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssetDescriptorRequestDto {
    pub raw_file_path: String,
    pub kind: String,
    pub asset_id: String,
    pub import_options: Option<CreateAssetImportOptionsDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpritesheetRulesetRequestDto {
    pub spritesheet_asset_key: String,
    pub ruleset_id: Option<String>,
}
