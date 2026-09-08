use amigo_plugin_api::CandidateStatus;
use amigo_scene::{RenderContributionsDocument, SceneVec2Document};
use amigo_sprite_2d_plugin::runtime::{
    collect_sprite_2d_candidates, extract_sprite_2d_renderables,
};
use amigo_sprite_2d_plugin::scene::Sprite2dDocument;

#[test]
fn sprite_waterfall_runs_from_manifest_and_document_to_render_candidate() {
    let manifest = amigo_plugin_manifest::parse_plugin_manifest_str(include_str!("../plugin.toml"))
        .expect("sprite plugin manifest should parse");
    assert_eq!(manifest.id.0, "amigo.gfx.sprite-2d");
    assert!(manifest
        .capabilities
        .provides
        .iter()
        .any(|capability| capability.id.0 == "gfx.sprite.2d" && capability.version == 1));
    assert!(manifest
        .targets
        .writes
        .iter()
        .any(|target| target.0 == "SceneColor"));
    assert!(manifest
        .targets
        .contributes
        .iter()
        .any(|target| target.0 == "SceneDepth"));

    let document = Sprite2dDocument {
        entity_name: "waterfall-sprite".to_owned(),
        render_layer: "world".to_owned(),
        texture: "sprite.png".to_owned(),
        size: SceneVec2Document { x: 32.0, y: 32.0 },
        sheet: None,
        animation: None,
        visual_maps: None,
        render_contributions: RenderContributionsDocument::default(),
        material: None,
        z_index: 0.25,
        opacity: 1.0,
        visible: true,
    };

    let candidates = collect_sprite_2d_candidates(&[document]);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].status, CandidateStatus::Active);
    assert!(candidates[0]
        .target_ids
        .iter()
        .any(|target| target.0 == "SceneColor"));

    let renderables = extract_sprite_2d_renderables(&candidates);
    assert_eq!(renderables.len(), 1);
    assert_eq!(renderables[0].status, CandidateStatus::Active);
}
