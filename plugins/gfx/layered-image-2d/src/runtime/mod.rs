use crate::api::LayeredImage2dCandidate;
use crate::scene::LayeredImage2dDocument;

pub fn collect_layered_image_2d_candidates(
    documents: &[LayeredImage2dDocument],
) -> Vec<LayeredImage2dCandidate> {
    documents
        .iter()
        .map(|document| {
            LayeredImage2dCandidate::active(document.entity_name.clone(), document.layers.clone())
        })
        .collect()
}
