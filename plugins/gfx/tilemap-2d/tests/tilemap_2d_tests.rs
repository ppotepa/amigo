use amigo_tilemap_2d_plugin::participation::adapters::focus_depth::tilemap_to_focus_depth;
use amigo_tilemap_2d_plugin::runtime::collect_tilemap_2d_candidates;
use amigo_tilemap_2d_plugin::scene::{
    tilemap_2d_scene_descriptor, Tilemap2dDocument,
};

#[test]
fn tilemap_collects_renderable_candidate_and_focus_depth_adapter() {
    let candidate = collect_tilemap_2d_candidates(&[Tilemap2dDocument {
        entity_name: "ground".to_owned(),
        render_layer: "world".to_owned(),
    }])
    .remove(0);

    assert_eq!(candidate.entity_name, "ground");
    assert!(matches!(
        tilemap_to_focus_depth(&candidate),
        amigo_focus_depth_plugin::api::FocusDepthCoverage2d::RenderLayer { .. }
    ));
}

#[test]
fn tilemap_plugin_owns_scene_descriptor() {
    let descriptor = tilemap_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.gfx.tilemap-2d.TileMap2D");
}
