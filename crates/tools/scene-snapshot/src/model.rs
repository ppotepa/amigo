use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneSnapshotRequest {
    pub mod_id: String,
    pub scene_id: String,
    pub mod_root: Option<PathBuf>,
    pub scene_path: Option<PathBuf>,
    pub width: u32,
    pub height: u32,
    pub mode: SceneSnapshotMode,
}

impl SceneSnapshotRequest {
    pub fn new(
        mod_id: impl Into<String>,
        scene_id: impl Into<String>,
        width: u32,
        height: u32,
        mode: SceneSnapshotMode,
    ) -> Self {
        Self {
            mod_id: mod_id.into(),
            scene_id: scene_id.into(),
            mod_root: None,
            scene_path: None,
            width,
            height,
            mode,
        }
    }

    pub fn with_paths(
        mut self,
        mod_root: impl Into<PathBuf>,
        scene_path: impl Into<PathBuf>,
    ) -> Self {
        self.mod_root = Some(mod_root.into());
        self.scene_path = Some(scene_path.into());
        self
    }

    pub fn cache_key(&self) -> String {
        format!(
            "{}:{}:{}x{}:{:?}:{}",
            self.mod_id,
            self.scene_id,
            self.width,
            self.height,
            self.mode,
            self.scene_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default()
        )
    }

    pub fn warmup_frames(&self) -> u32 {
        match &self.mode {
            SceneSnapshotMode::Static => 0,
            SceneSnapshotMode::EnginePreview => 1,
            SceneSnapshotMode::AfterOnEnter => 0,
            SceneSnapshotMode::AfterWarmupFrames(frames) => *frames,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SceneSnapshotMode {
    Static,
    EnginePreview,
    AfterOnEnter,
    AfterWarmupFrames(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneSnapshotImage {
    pub request: SceneSnapshotRequest,
    pub width: u32,
    pub height: u32,
    pub pixels_rgba8: Vec<u8>,
    pub diagnostic_label: String,
}

impl SceneSnapshotImage {
    pub fn cache_key(&self) -> String {
        self.request.cache_key()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneSnapshotError {
    pub message: String,
}

impl SceneSnapshotError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
