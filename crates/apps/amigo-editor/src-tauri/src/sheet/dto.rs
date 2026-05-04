use serde::{Deserialize, Serialize};

use crate::dto::EditorDiagnosticDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetResourceDto {
    pub resource_uri: String,
    pub absolute_path: String,
    pub relative_path: String,
    pub kind: SheetKind,
    pub schema_version: u32,
    pub source_schema_kind: SheetSourceSchemaKind,
    pub id: String,
    pub label: String,
    pub image_path: String,
    pub image_absolute_path: String,
    pub image_exists: bool,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub declared_image_width: Option<u32>,
    pub declared_image_height: Option<u32>,
    pub cell_width: u32,
    pub cell_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub count: u32,
    pub margin_x: u32,
    pub margin_y: u32,
    pub spacing_x: u32,
    pub spacing_y: u32,
    pub tileset: Option<TileSetPayloadDto>,
    pub animations: Option<Vec<SpriteAnimationDto>>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SheetKind {
    Tileset,
    Spritesheet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SheetSourceSchemaKind {
    Descriptor,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSetPayloadDto {
    pub defaults: TileSetDefaultsDto,
    pub tiles: Vec<TileMetadataDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSetDefaultsDto {
    pub collision: String,
    pub damageable: bool,
}

impl Default for TileSetDefaultsDto {
    fn default() -> Self {
        Self {
            collision: "solid".to_owned(),
            damageable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileMetadataDto {
    pub key: String,
    pub id: u32,
    pub role: Option<String>,
    pub name: Option<String>,
    pub category: Option<String>,
    pub collision: Option<String>,
    pub damageable: Option<bool>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteAnimationDto {
    pub id: String,
    pub frames: Vec<u32>,
    pub fps: Option<f32>,
    pub looping: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilemapResourceDto {
    pub resource_uri: String,
    pub absolute_path: String,
    pub relative_path: String,
    pub schema_version: u32,
    pub id: String,
    pub label: String,
    pub tileset_resource_uri: Option<String>,
    pub width: u32,
    pub height: u32,
    pub origin_offset_x: i32,
    pub origin_offset_y: i32,
    pub cells: Vec<TilemapCellDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilemapCellDto {
    pub x: u32,
    pub y: u32,
    pub tile_id: u32,
}
