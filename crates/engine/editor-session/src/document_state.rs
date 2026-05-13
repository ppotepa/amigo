#[derive(Debug, Clone, Default)]
pub struct SceneDocumentState {
    pub id: String,
    pub source_path: Option<String>,
    pub dirty: bool,
}

