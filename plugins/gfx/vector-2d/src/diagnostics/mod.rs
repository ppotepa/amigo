use crate::api::Vector2dRenderableCandidate;

pub fn format_vector_2d_candidates(candidates: &[Vector2dRenderableCandidate]) -> String {
    format!("vector_2d.candidates count={}", candidates.len())
}
