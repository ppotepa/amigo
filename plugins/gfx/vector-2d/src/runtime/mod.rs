use crate::api::Vector2dRenderableCandidate;
use crate::scene::Vector2dDocument;

pub fn collect_vector_2d_candidates(
    documents: &[Vector2dDocument],
) -> Vec<Vector2dRenderableCandidate> {
    documents
        .iter()
        .map(|document| {
            Vector2dRenderableCandidate::active(
                document.entity_name.clone(),
                document.render_layer.clone(),
            )
        })
        .collect()
}
