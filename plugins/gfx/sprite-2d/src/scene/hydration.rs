use crate::api::{Sprite2dRenderableCandidate, Sprite2dRenderResponse};

use super::Sprite2dDocument;

pub fn sprite_candidate_from_document(document: &Sprite2dDocument) -> Sprite2dRenderableCandidate {
    Sprite2dRenderableCandidate::active(
        document.entity_name.clone(),
        document.render_layer.clone(),
        Sprite2dRenderResponse {
            visible: document.visible,
            opacity: document.opacity,
        },
    )
}
