#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewRequest {
    pub mod_id: String,
    pub scene_id: String,
}

impl PreviewRequest {
    pub fn new(mod_id: impl Into<String>, scene_id: impl Into<String>) -> Self {
        Self {
            mod_id: mod_id.into(),
            scene_id: scene_id.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewSceneInfo {
    pub mod_id: String,
    pub scene_id: String,
    pub scene_label: String,
    pub scene_count: usize,
}

impl PreviewSceneInfo {
    pub fn new(
        mod_id: impl Into<String>,
        scene_id: impl Into<String>,
        scene_label: impl Into<String>,
        scene_count: usize,
    ) -> Self {
        Self {
            mod_id: mod_id.into(),
            scene_id: scene_id.into(),
            scene_label: scene_label.into(),
            scene_count,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewState {
    Empty,
    Loading {
        request: PreviewRequest,
        frames_remaining: u8,
    },
    ReadyPlaceholder {
        info: PreviewSceneInfo,
    },
    Error {
        request: Option<PreviewRequest>,
        message: String,
    },
}

impl PreviewState {
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }
}
