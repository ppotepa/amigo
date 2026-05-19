use amigo_plugin_api::TargetId;

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

pub fn target_id_for_visual_kind_name(kind: &str) -> Option<TargetId> {
    match kind {
        "SceneHighlight" | "scene_highlight" => Some(super::scene_highlight_target_id()),
        "SceneEmissive" | "scene_emissive" => Some(super::scene_emissive_target_id()),
        _ => None,
    }
}
