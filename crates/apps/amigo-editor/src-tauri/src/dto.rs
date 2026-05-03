use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDiagnosticDto {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum EditorStatus {
    Valid,
    Warning,
    Error,
    MissingDependency,
    InvalidManifest,
    MissingSceneFile,
    PreviewFailed,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum PreviewStatus {
    Missing,
    Queued,
    Rendering,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorModSummaryDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub root_path: String,
    pub dependencies: Vec<String>,
    pub missing_dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub scene_count: usize,
    pub visible_scene_count: usize,
    pub status: EditorStatus,
    pub diagnostics: Vec<EditorDiagnosticDto>,
    pub last_modified: Option<String>,
    pub project_cache_id: String,
    pub preview_status: PreviewStatus,
    pub content_summary: EditorContentSummaryDto,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorContentSummaryDto {
    pub scenes: usize,
    pub scene_yaml: usize,
    pub scripts: usize,
    pub textures: usize,
    pub spritesheets: usize,
    pub audio: usize,
    pub tilemaps: usize,
    pub tilesets: usize,
    pub packages: usize,
    pub unknown_files: usize,
    pub total_files: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneSummaryDto {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub path: String,
    pub document_path: String,
    pub script_path: String,
    pub launcher_visible: bool,
    pub status: EditorStatus,
    pub preview_cache_key: String,
    pub preview_image_url: Option<String>,
    pub preview_fps: u32,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorModDetailsDto {
    #[serde(flatten)]
    pub summary: EditorModSummaryDto,
    pub scenes: Vec<EditorSceneSummaryDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePreviewDto {
    pub mod_id: String,
    pub scene_id: String,
    pub status: PreviewStatus,
    pub fps: u32,
    pub frame_count: u32,
    pub image_url: Option<String>,
    pub frame_urls: Vec<String>,
    pub width: u32,
    pub height: u32,
    pub duration_ms: u32,
    pub generated_at: Option<String>,
    pub source_hash: String,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneEntityDto {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub groups: Vec<String>,
    pub visible: bool,
    pub simulation_enabled: bool,
    pub collision_enabled: bool,
    pub has_transform2: bool,
    pub has_transform3: bool,
    pub property_count: usize,
    pub component_count: usize,
    pub component_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneHierarchyDto {
    pub mod_id: String,
    pub scene_id: String,
    pub scene_label: String,
    pub entity_count: usize,
    pub component_count: usize,
    pub entities: Vec<EditorSceneEntityDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorProjectFileDto {
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub kind: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub children: Vec<EditorProjectFileDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorProjectTreeDto {
    pub mod_id: String,
    pub root_path: String,
    pub total_files: usize,
    pub root: EditorProjectFileDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorProjectStructureTreeDto {
    pub mod_id: String,
    pub root_path: String,
    pub root: EditorProjectStructureNodeDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorProjectStructureNodeDto {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub icon: String,
    pub status: Option<String>,
    pub count: Option<usize>,
    pub path: Option<String>,
    pub expected_path: Option<String>,
    pub exists: bool,
    pub empty: bool,
    pub ghost: bool,
    pub file: Option<EditorProjectFileDto>,
    pub scene: Option<EditorSceneSummaryDto>,
    pub children: Vec<EditorProjectStructureNodeDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorProjectFileContentDto {
    pub mod_id: String,
    pub path: String,
    pub relative_path: String,
    pub kind: String,
    pub language: String,
    pub size_bytes: u64,
    pub content: String,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenModResultDto {
    pub mod_id: String,
    pub root_path: String,
    pub session_id: String,
    pub created_at: String,
    pub selected_scene_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSettingsDto {
    #[serde(default = "default_settings_version")]
    pub settings_version: u32,
    pub mods_root: Option<String>,
    pub cache_root_override: Option<String>,
    pub active_theme_id: String,
    #[serde(default = "default_active_font_id")]
    pub active_font_id: String,
    pub last_opened_mod_id: Option<String>,
}

fn default_settings_version() -> u32 {
    1
}

fn default_active_font_id() -> String {
    "source-sans-3".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheProjectInfoDto {
    pub project_cache_id: String,
    pub mod_id: String,
    pub display_name: String,
    pub root_path: String,
    pub last_seen_at: String,
    pub project_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInfoDto {
    pub cache_root: String,
    pub cache_root_mode: String,
    pub total_size_bytes: u64,
    pub project_count: usize,
    pub projects: Vec<CacheProjectInfoDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAliasesDto {
    pub display_names: Vec<String>,
    pub mod_ids: Vec<String>,
    pub root_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIndexEntryDto {
    pub project_cache_id: String,
    pub last_known_display_name: String,
    pub last_known_mod_id: String,
    pub last_known_root_path: String,
    pub last_seen_at: String,
    pub aliases: ProjectAliasesDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIndexDto {
    pub version: u32,
    pub projects: Vec<ProjectIndexEntryDto>,
}

impl Default for ProjectIndexDto {
    fn default() -> Self {
        Self {
            version: 1,
            projects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachePolicyDto {
    pub max_preview_cache_bytes: Option<u64>,
    pub max_age_days: Option<u32>,
    pub auto_cleanup_enabled: bool,
}

impl Default for CachePolicyDto {
    fn default() -> Self {
        Self {
            max_preview_cache_bytes: None,
            max_age_days: None,
            auto_cleanup_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheMaintenanceResultDto {
    pub removed_entries: usize,
    pub removed_bytes: u64,
    pub remaining_preview_bytes: u64,
    pub orphaned_projects_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSessionDto {
    pub session_id: String,
    pub mod_id: String,
    pub root_path: String,
    pub created_at: String,
    pub selected_scene_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePreviewFrameGeneratedDto {
    pub mod_id: String,
    pub scene_id: String,
    pub current: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorWindowInfoDto {
    pub label: String,
    pub kind: String,
    pub session_id: Option<String>,
    pub focused: bool,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorWindowRegistryDto {
    pub windows: Vec<EditorWindowInfoDto>,
}
