use camino::Utf8PathBuf;

/// Options used when constructing a runtime session.
#[derive(Debug, Clone)]
pub struct RuntimeSessionOptions {
    pub project_root: Utf8PathBuf,
    pub mod_id: Option<String>,
    pub scene_id: Option<String>,
    pub dev_mode: bool,
}

/// Describes how a runtime session will be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSessionProfile {
    Game,
    EditorPreview,
    HeadlessValidation,
    SceneThumbnail,
    Test,
}
