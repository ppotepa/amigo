use crate::api::Text2dRenderableCandidate;
use crate::scene::Text2dDocument;

pub fn collect_text_2d_candidates(documents: &[Text2dDocument]) -> Vec<Text2dRenderableCandidate> {
    documents
        .iter()
        .map(|document| {
            Text2dRenderableCandidate::active(
                document.entity_name.clone(),
                document.render_layer.clone(),
            )
        })
        .collect()
}
