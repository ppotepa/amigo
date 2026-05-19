use crate::api::Sprite2dRenderableCandidate;
use crate::scene::{sprite_candidate_from_document, Sprite2dDocument};

pub fn collect_sprite_2d_candidates(
    documents: &[Sprite2dDocument],
) -> Vec<Sprite2dRenderableCandidate> {
    documents.iter().map(sprite_candidate_from_document).collect()
}
