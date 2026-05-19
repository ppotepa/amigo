use amigo_plugin_api::CandidateStatus;
use amigo_sprite_2d_plugin::participation::adapters::{
    camera_optics::sprite_coverage_to_camera_optics,
    focus_depth::sprite_coverage_to_focus_depth,
    shutter_motion::sprite_coverage_to_shutter_motion,
};
use amigo_sprite_2d_plugin::runtime::collect_sprite_2d_candidates;
use amigo_sprite_2d_plugin::scene::{
    sprite_2d_scene_descriptor, Sprite2dDocument,
};

#[test]
fn sprite_document_collects_active_renderable_candidate() {
    let candidates = collect_sprite_2d_candidates(&[Sprite2dDocument {
        entity_name: "hero".to_owned(),
        render_layer: "world".to_owned(),
        texture: "hero.png".to_owned(),
        opacity: 1.0,
        visible: true,
    }]);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].status, CandidateStatus::Active);
    assert!(candidates[0]
        .target_ids
        .iter()
        .any(|target| target.0 == "SceneColor"));
}

#[test]
fn sprite_contributions_are_adapter_mapped() {
    let candidate = collect_sprite_2d_candidates(&[Sprite2dDocument {
        entity_name: "hero".to_owned(),
        render_layer: "world".to_owned(),
        texture: "hero.png".to_owned(),
        opacity: 1.0,
        visible: true,
    }])
    .remove(0);

    assert!(sprite_coverage_to_camera_optics(&candidate.coverage).is_some());
    assert!(sprite_coverage_to_focus_depth(&candidate.coverage).is_some());
    assert!(sprite_coverage_to_shutter_motion(&candidate.coverage).is_some());
}

#[test]
fn sprite_plugin_owns_scene_descriptor() {
    let descriptor = sprite_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.gfx.sprite-2d.Sprite2D");
}
