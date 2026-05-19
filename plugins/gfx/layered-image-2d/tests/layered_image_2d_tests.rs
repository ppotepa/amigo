use amigo_layered_image_2d_plugin::api::LayeredImage2dLayer;
use amigo_layered_image_2d_plugin::participation::adapters::focus_depth::layered_image_layer_to_focus_depth;
use amigo_layered_image_2d_plugin::runtime::collect_layered_image_2d_candidates;
use amigo_layered_image_2d_plugin::scene::{
    layered_image_2d_scene_descriptor, LayeredImage2dDocument,
};

#[test]
fn layered_image_collects_renderable_candidate_and_focus_depth_adapter() {
    let layer = LayeredImage2dLayer {
        id: "near".to_owned(),
        distance_m: Some(2.0),
        blur_scale: 1.0,
    };
    let candidate = collect_layered_image_2d_candidates(&[LayeredImage2dDocument {
        entity_name: "alley".to_owned(),
        layers: vec![layer.clone()],
    }])
    .remove(0);

    assert_eq!(candidate.entity_name, "alley");
    assert!(
        layered_image_layer_to_focus_depth(&layer)
            .0
            .starts_with("focus-depth.distance.")
    );
}

#[test]
fn layered_image_plugin_owns_scene_descriptor() {
    let descriptor = layered_image_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.gfx.layered-image-2d.LayeredImage2D"
    );
}
