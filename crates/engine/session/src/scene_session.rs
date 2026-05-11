use std::path::PathBuf;

/// Host-independent scene session state.
///
/// This type is intentionally lightweight in Etap 3. It establishes the
/// reusable scene-session boundary without moving app-owned scene handlers yet.
/// Later passes should move scene loading, hydration, command dispatch and
/// scene cleanup behind this boundary.
#[derive(Debug, Clone, Default)]
pub struct SceneSession {
    loaded_scene_document: Option<SceneSessionLoadedDocument>,
}

impl SceneSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns metadata for the loaded authored scene document, if any.
    pub fn loaded_scene_document(&self) -> Option<&SceneSessionLoadedDocument> {
        self.loaded_scene_document.as_ref()
    }

    /// Returns the active loaded scene id, if known.
    pub fn active_scene_id(&self) -> Option<&str> {
        self.loaded_scene_document
            .as_ref()
            .map(|document| document.scene_id.as_str())
    }

    /// Mark a scene document as loaded by the session.
    ///
    /// This is used by the app bootstrap adapter during the migration. Once
    /// scene loading moves into `amigo-session`, this method should be called
    /// by the session-owned loader instead.
    pub fn mark_loaded_scene_document(&mut self, document: SceneSessionLoadedDocument) {
        self.loaded_scene_document = Some(document);
    }

    /// Clear session-owned scene metadata.
    ///
    /// This does not yet clear runtime scene services. That responsibility
    /// still lives in `amigo-app` until the scene lifecycle migration pass.
    pub fn clear_loaded_scene_document(&mut self) {
        self.loaded_scene_document = None;
    }
}

/// Summary of an authored scene document loaded into a runtime session.
///
/// This mirrors only host-independent metadata. App/editor-specific summaries
/// should be converted into this type instead of being stored in
/// `amigo-session` directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSessionLoadedDocument {
    pub source_mod: String,
    pub scene_id: String,
    pub relative_path: PathBuf,
    pub entity_count: usize,
    pub component_count: usize,
    pub transition_count: usize,
}

impl SceneSessionLoadedDocument {
    pub fn new(
        source_mod: impl Into<String>,
        scene_id: impl Into<String>,
        relative_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            scene_id: scene_id.into(),
            relative_path: relative_path.into(),
            entity_count: 0,
            component_count: 0,
            transition_count: 0,
        }
    }

    pub fn with_counts(
        mut self,
        entity_count: usize,
        component_count: usize,
        transition_count: usize,
    ) -> Self {
        self.entity_count = entity_count;
        self.component_count = component_count;
        self.transition_count = transition_count;
        self
    }
}
