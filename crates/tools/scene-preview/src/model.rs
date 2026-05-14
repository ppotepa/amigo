use amigo_tool_scene_snapshot::SceneSnapshotImage;

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

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewSnapshot {
    pub info: PreviewSceneInfo,
    pub source_path: Option<String>,
    pub entities_count: usize,
    pub draw_items: Vec<PreviewDrawItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreviewDrawItem {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: PreviewColor,
        label: Option<String>,
    },
    Circle {
        x: f32,
        y: f32,
        r: f32,
        color: PreviewColor,
        label: Option<String>,
    },
    Label {
        x: f32,
        y: f32,
        text: String,
        color: PreviewColor,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl PreviewColor {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreviewState {
    Empty,
    Loading {
        request: PreviewRequest,
        frames_remaining: u8,
    },
    ReadyPlaceholder {
        info: PreviewSceneInfo,
    },
    ReadySnapshot {
        snapshot: PreviewSnapshot,
    },
    ReadyRendered {
        image: SceneSnapshotImage,
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
