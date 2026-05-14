use glam::UVec2;

/// Per-frame input supplied by a concrete host to a runtime session.
#[derive(Debug, Clone)]
pub struct RuntimeFrameInput {
    pub dt_seconds: f32,
    pub viewport_size: UVec2,
}

/// Per-frame high-level output produced by a runtime session.
#[derive(Debug, Clone, Default)]
pub struct RuntimeFrameOutput {
    pub requested_scene_transition: bool,
    pub diagnostics_changed: bool,
}

/// Host-independent render target metadata.
#[derive(Debug, Clone)]
pub struct RenderTargetInfo {
    pub size: UVec2,
    pub scale_factor: f32,
}
