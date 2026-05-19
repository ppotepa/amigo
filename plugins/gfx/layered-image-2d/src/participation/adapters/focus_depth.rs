use amigo_plugin_api::TargetId;

use crate::api::LayeredImage2dLayer;

pub fn layered_image_layer_to_focus_depth(layer: &LayeredImage2dLayer) -> TargetId {
    match layer.distance_m {
        Some(meters) => TargetId(format!("focus-depth.distance.{meters:.3}")),
        None => TargetId(format!("focus-depth.layer.{}", layer.id)),
    }
}
