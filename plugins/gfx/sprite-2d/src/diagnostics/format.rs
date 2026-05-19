use crate::api::Sprite2dRenderableCandidate;

pub fn format_sprite_2d_candidates(candidates: &[Sprite2dRenderableCandidate]) -> String {
    if candidates.is_empty() {
        return "sprite_2d.candidates: none".to_owned();
    }

    candidates
        .iter()
        .map(|candidate| {
            format!(
                "entity={} status={:?} reason={} targets={}",
                candidate.entity_name,
                candidate.status,
                candidate.reason,
                candidate
                    .target_ids
                    .iter()
                    .map(|target| target.0.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
