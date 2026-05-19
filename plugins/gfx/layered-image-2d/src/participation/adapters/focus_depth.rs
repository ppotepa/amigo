use amigo_focus_depth_plugin::api::FocusDepthCoverage2d;

use crate::api::LayeredImage2dLayer;

pub fn layered_image_layer_to_focus_depth(layer: &LayeredImage2dLayer) -> FocusDepthCoverage2d {
    match layer.distance_m {
        Some(meters) => FocusDepthCoverage2d::Distance { meters },
        None => FocusDepthCoverage2d::RenderLayer {
            layer_id: layer.id.clone(),
        },
    }
}
