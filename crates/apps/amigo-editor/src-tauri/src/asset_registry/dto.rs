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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMigrationPlanDto {
    pub session_id: String,
    pub mod_id: String,
    pub root_path: String,
    pub entries: Vec<AssetMigrationEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMigrationEntryDto {
    pub action: String,
    pub from_path: Option<String>,
    pub to_path: Option<String>,
    pub asset_kind: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMigrationResultDto {
    pub dry_run: bool,
    pub applied_entries: usize,
    pub report_path: Option<String>,
}
