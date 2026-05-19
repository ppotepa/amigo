use crate::api::Tilemap2dCandidate;
use crate::scene::Tilemap2dDocument;

pub fn collect_tilemap_2d_candidates(documents: &[Tilemap2dDocument]) -> Vec<Tilemap2dCandidate> {
    documents
        .iter()
        .map(|document| {
            Tilemap2dCandidate::active(document.entity_name.clone(), document.render_layer.clone())
        })
        .collect()
}
