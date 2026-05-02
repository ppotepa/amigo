use crate::{PreviewRequest, PreviewSceneInfo, PreviewState};

#[derive(Debug)]
pub struct ScenePreviewController {
    state: PreviewState,
    loading_frames: u8,
    pending_info: Option<PreviewSceneInfo>,
}

impl Default for ScenePreviewController {
    fn default() -> Self {
        Self::new()
    }
}

impl ScenePreviewController {
    pub fn new() -> Self {
        Self {
            state: PreviewState::Empty,
            loading_frames: 24,
            pending_info: None,
        }
    }

    pub fn state(&self) -> &PreviewState {
        &self.state
    }

    pub fn request_placeholder(&mut self, info: PreviewSceneInfo) {
        let request = PreviewRequest::new(info.mod_id.clone(), info.scene_id.clone());
        self.pending_info = Some(info);
        self.state = PreviewState::Loading {
            request,
            frames_remaining: self.loading_frames,
        };
    }

    pub fn clear(&mut self) {
        self.pending_info = None;
        self.state = PreviewState::Empty;
    }

    pub fn set_error(&mut self, request: Option<PreviewRequest>, message: impl Into<String>) {
        self.pending_info = None;
        self.state = PreviewState::Error {
            request,
            message: message.into(),
        };
    }

    pub fn tick(&mut self) -> bool {
        let state = std::mem::replace(&mut self.state, PreviewState::Empty);
        match state {
            PreviewState::Loading {
                request,
                mut frames_remaining,
            } => {
                if frames_remaining > 0 {
                    frames_remaining -= 1;
                }

                if frames_remaining == 0 {
                    self.state = if let Some(info) = self.pending_info.take() {
                        PreviewState::ReadyPlaceholder { info }
                    } else {
                        PreviewState::Error {
                            request: Some(request),
                            message: String::from("Preview finished loading without scene info"),
                        }
                    };
                } else {
                    self.state = PreviewState::Loading {
                        request,
                        frames_remaining,
                    };
                }

                true
            }
            other => {
                self.state = other;
                false
            }
        }
    }
}
