use amigo_plugin_api::CandidateStatus;
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemRegistry};
use amigo_scene::{
    ComponentGraphProviderRegistry, ComponentHydratorRegistry, ComponentSchemaRegistry,
    PluginSceneCommandHandlerRegistry, RuntimeSceneCommandHandlerRegistry, ScenePlugin,
};
use amigo_scripting_api::RuntimeScriptCommandHandlerRegistry;
use amigo_text_2d_plugin::participation::adapters::camera_optics::text_coverage_to_camera_optics;
use amigo_text_2d_plugin::runtime::collect_text_2d_candidates;
use amigo_text_2d_plugin::scene::{
    text_2d_scene_descriptor, Text2dDocument,
};
use amigo_text_2d_plugin::Text2dPlugin;

#[test]
fn text_document_collects_glyph_candidate() {
    let candidate = collect_text_2d_candidates(&[Text2dDocument {
        entity_name: "title".to_owned(),
        render_layer: "ui".to_owned(),
        content: "Amigo".to_owned(),
        font: "test/font".to_owned(),
        bounds: amigo_scene::SceneVec2Document { x: 256.0, y: 64.0 },
        style: Default::default(),
        render_contributions: Default::default(),
        z_index: 0.0,
        material: None,
    }])
    .remove(0);

    assert_eq!(candidate.status, CandidateStatus::Active);
    assert!(candidate
        .target_ids
        .iter()
        .any(|target| target.0 == "SceneColor"));
    assert!(matches!(
        text_coverage_to_camera_optics(&candidate.coverage),
        amigo_camera_optics_plugin::api::CameraOpticalCoverage2d::Glyphs { .. }
    ));
}

#[test]
fn text_plugin_owns_scene_descriptor() {
    let descriptor = text_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.gfx.text-2d.Text2D");
}

#[test]
fn text_plugin_registers_schema_provider_and_hydrator() {
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
    Text2dPlugin
        .register(&mut registry)
        .expect("text plugin should register");

    let schemas = registry
        .resolve::<ComponentSchemaRegistry>()
        .expect("component schema registry should exist");
    let hydrators = registry
        .resolve::<ComponentHydratorRegistry>()
        .expect("component hydrator registry should exist");
    let graph_providers = registry
        .resolve::<ComponentGraphProviderRegistry>()
        .expect("component graph provider registry should exist");
    let plugin_scene_handlers = registry
        .resolve::<PluginSceneCommandHandlerRegistry>()
        .expect("plugin scene command handler registry should exist");

    assert!(schemas
        .known_component_types()
        .iter()
        .any(|id| id == "amigo.gfx.text-2d.Text2D"));
    assert!(hydrators.provider_ids().contains(&"amigo.gfx.text-2d"));
    assert!(graph_providers.provider_ids().contains(&"amigo.gfx.text-2d"));
    assert!(plugin_scene_handlers
        .handler_for("amigo.gfx.text-2d.scene-command.Text2D")
        .is_some());
}
