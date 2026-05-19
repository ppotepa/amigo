use crate::api::Text2dRenderableCandidate;

pub fn format_text_2d_candidates(candidates: &[Text2dRenderableCandidate]) -> String {
    format!("text_2d.candidates count={}", candidates.len())
}
