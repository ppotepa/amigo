use crate::api::Sprite2dRenderableCandidate;

pub fn extract_sprite_2d_renderables(
    candidates: &[Sprite2dRenderableCandidate],
) -> Vec<Sprite2dRenderableCandidate> {
    candidates
        .iter()
        .filter(|candidate| candidate.status == amigo_plugin_api::CandidateStatus::Active)
        .cloned()
        .collect()
}
