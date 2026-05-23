use amigo_plugin_api::CandidateStatus;
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemRegistry};
use amigo_scene::{
    ComponentHydratorRegistry, ComponentSchemaRegistry, PluginSceneCommandHandlerRegistry,
    RuntimeSceneCommandHandlerRegistry, ScenePlugin,
};
use amigo_sprite_2d_plugin::participation::adapters::{
    camera_optics::sprite_coverage_to_camera_optics,
    focus_depth::sprite_coverage_to_focus_depth,
    shutter_motion::sprite_coverage_to_shutter_motion,
};
use amigo_sprite_2d_plugin::runtime::collect_sprite_2d_candidates;
use amigo_sprite_2d_plugin::SpritePlugin;
use amigo_sprite_2d_plugin::scene::{
    sprite_2d_scene_descriptor, Sprite2dDocument,
};
use amigo_scripting_api::RuntimeScriptCommandHandlerRegistry;
use amigo_scene::RenderContributionsDocument;
use amigo_scene::SceneVec2Document;

#[test]
fn sprite_document_collects_active_renderable_candidate() {
    let candidates = collect_sprite_2d_candidates(&[Sprite2dDocument {
        entity_name: "hero".to_owned(),
        render_layer: "world".to_owned(),
        texture: "hero.png".to_owned(),
        size: SceneVec2Document { x: 64.0, y: 64.0 },
        sheet: None,
        animation: None,
        visual_maps: None,
        render_contributions: RenderContributionsDocument::default(),
        material: None,
        z_index: 0.0,
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
        size: SceneVec2Document { x: 64.0, y: 64.0 },
        sheet: None,
        animation: None,
        visual_maps: None,
        render_contributions: RenderContributionsDocument::default(),
        material: None,
        z_index: 0.0,
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

#[test]
fn sprite_plugin_registers_schema_provider_and_hydrator() {
    let mut registry = ServiceRegistry::default();
    registry
        .register(RuntimeSceneCommandHandlerRegistry::new())
        .expect("scene command registry should register");
    registry
        .register(RuntimeScriptCommandHandlerRegistry::new())
        .expect("script command registry should register");
    registry
        .register(SystemRegistry::default())
        .expect("system registry should register");

    ScenePlugin
        .register(&mut registry)
        .expect("scene plugin should register");
    SpritePlugin
        .register(&mut registry)
        .expect("sprite plugin should register");

    let schemas = registry
        .resolve::<ComponentSchemaRegistry>()
        .expect("component schema registry should exist");
    let hydrators = registry
        .resolve::<ComponentHydratorRegistry>()
        .expect("component hydrator registry should exist");
    let plugin_scene_handlers = registry
        .resolve::<PluginSceneCommandHandlerRegistry>()
        .expect("plugin scene command handler registry should exist");

    assert!(schemas
        .known_component_types()
        .iter()
        .any(|id| id == "amigo.gfx.sprite-2d.Sprite2D"));
    assert!(hydrators.provider_ids().contains(&"amigo.gfx.sprite-2d"));
    assert!(plugin_scene_handlers
        .handler_for("amigo.gfx.sprite-2d.scene-command.Sprite2D")
        .is_some());
}
