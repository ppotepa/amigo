use amigo_render_api::VisualSourceKind2d;

use crate::WgpuOffscreenTarget;

#[derive(Default)]
pub(crate) struct WgpuVisualSourceTargets2d {
    pub layer_mask: Option<WgpuOffscreenTarget>,
    pub layer_roles: Option<WgpuOffscreenTarget>,
    pub scene_normal: Option<WgpuOffscreenTarget>,
    pub scene_wetness: Option<WgpuOffscreenTarget>,
    pub scene_highlight: Option<WgpuOffscreenTarget>,
    pub scene_emissive: Option<WgpuOffscreenTarget>,
    pub scene_motion: Option<WgpuOffscreenTarget>,
}

impl WgpuVisualSourceTargets2d {
    pub fn get(&self, kind: VisualSourceKind2d) -> Option<&WgpuOffscreenTarget> {
        match kind {
            VisualSourceKind2d::LayerMask => self.layer_mask.as_ref(),
            VisualSourceKind2d::SceneNormal => self.scene_normal.as_ref(),
            VisualSourceKind2d::SceneWetness => self.scene_wetness.as_ref(),
            VisualSourceKind2d::SceneHighlight => self.scene_highlight.as_ref(),
            VisualSourceKind2d::SceneEmissive => self.scene_emissive.as_ref(),
            VisualSourceKind2d::SceneMotion => self.scene_motion.as_ref(),
            VisualSourceKind2d::SceneColor
            | VisualSourceKind2d::SceneDepth
            | VisualSourceKind2d::Debug => None,
        }
    }
}
