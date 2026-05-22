use amigo_plugin_api::{StandardTarget, TargetId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CameraOpticalRenderTargetPlan {
    pub target: TargetId,
    pub accepts_color_candidates: bool,
    pub accepts_texture_candidates: bool,
}

impl CameraOpticalRenderTargetPlan {
    pub fn for_visual_kind_name(kind: &str) -> Option<Self> {
        let target = target_id_for_visual_kind_name(kind)?;
        Some(Self {
            target,
            accepts_color_candidates: true,
            accepts_texture_candidates: true,
        })
    }

    pub fn is_camera_optics_target_name(kind: &str) -> bool {
        target_id_for_visual_kind_name(kind).is_some()
    }
}

pub fn scene_highlight_target_id() -> TargetId {
    StandardTarget::SceneHighlight.id()
}

pub fn scene_emissive_target_id() -> TargetId {
    StandardTarget::SceneEmissive.id()
}

pub fn target_id_for_visual_kind_name(kind: &str) -> Option<TargetId> {
    match kind {
        "SceneHighlight" | "scene_highlight" => Some(scene_highlight_target_id()),
        "SceneEmissive" | "scene_emissive" => Some(scene_emissive_target_id()),
        _ => None,
    }
}
