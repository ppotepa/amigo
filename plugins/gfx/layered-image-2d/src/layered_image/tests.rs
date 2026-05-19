use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use amigo_2d_post_fx::PostFx2d;
use amigo_assets::{AssetCatalog, AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};
use amigo_math::{Transform2, Vec2};
use amigo_runtime_control::{ControlValue, RuntimeControlService};
use amigo_scene::{SceneCommand, SceneEntityId, SceneEvent, SceneEventQueue, SceneService};

use super::{
    LayeredImageBlendMode2d, LayeredImageDrawCommand, LayeredImageInstance,
    LayeredImageSceneCommandContext, LayeredImageSceneService, LayeredImageViewportFit2d,
    can_handle_layered_image_scene_command, handle_layered_image_scene_command,
    infer_layered_image_asset_from_prepared,
};

fn test_prepared_layered_image() -> PreparedAsset {
    PreparedAsset {
        key: AssetKey::new("test-mod/layered-images/test-scene"),
        source: AssetSourceKind::Mod("test-mod".to_owned()),
        resolved_path: PathBuf::from("layered-images/test-scene/layered-image.yml"),
        byte_len: 128,
        kind: PreparedAssetKind::LayeredImage2d,
        label: Some("Neon Alley".to_owned()),
        format: None,
        metadata: BTreeMap::from([
            ("canvas.width".to_owned(), "1672".to_owned()),
            ("canvas.height".to_owned(), "941".to_owned()),
            ("base.image".to_owned(), "base_albedo.png".to_owned()),
            ("layers.0.id".to_owned(), "windows".to_owned()),
            ("layers.0.label".to_owned(), "Windows".to_owned()),
            ("layers.0.image".to_owned(), "windows.png".to_owned()),
            ("layers.0.blend".to_owned(), "alpha".to_owned()),
            ("layers.0.default_opacity".to_owned(), "0.75".to_owned()),
        ]),
    }
}

#[test]
fn infers_layered_image_asset_from_prepared_metadata() {
    let mut prepared = test_prepared_layered_image();
    prepared.metadata.extend(BTreeMap::from([
        ("layers.0.id".to_owned(), "accent_light".to_owned()),
        ("layers.0.label".to_owned(), "Accent Light".to_owned()),
        ("layers.0.image".to_owned(), "light_001.png".to_owned()),
        ("layers.0.blend".to_owned(), "additive".to_owned()),
        ("layers.0.color".to_owned(), "#FF1493".to_owned()),
        ("layers.0.post_fx.kind".to_owned(), "blur".to_owned()),
        ("layers.0.post_fx.radius".to_owned(), "18.0".to_owned()),
        ("layers.0.post_fx.downsample".to_owned(), "0.5".to_owned()),
        ("layers.0.post_fx.intensity".to_owned(), "1.2".to_owned()),
    ]));

    let asset = infer_layered_image_asset_from_prepared(&prepared).unwrap();

    assert_eq!(asset.canvas_size, Vec2::new(1672.0, 941.0));
    assert_eq!(asset.base_image, "base_albedo.png");
    assert_eq!(asset.layers.len(), 1);
    assert_eq!(asset.layers[0].id, "accent_light");
    assert_eq!(
        asset.layers[0].blend_mode,
        LayeredImageBlendMode2d::Additive
    );
    assert_eq!(asset.layers[0].opacity, 0.75);
    let Some(post_fx) = asset.layers[0].post_fx.as_ref() else {
        panic!("layer should infer post-fx stack");
    };
    assert_eq!(post_fx.effects.len(), 1);
    match post_fx.effects[0] {
        PostFx2d::Blur(blur) => {
            assert_eq!(blur.radius, 18.0);
            assert_eq!(blur.downsample, 0.5);
            assert_eq!(blur.intensity, 1.2);
        }
        PostFx2d::EmbossEdges(_) => panic!("expected blur effect for this fixture"),
        PostFx2d::ColorQuantize(_)
        | PostFx2d::Crt(_)
        | PostFx2d::DirtyBloom(_)
        | PostFx2d::FilmNoise(_)
        | PostFx2d::LensDroplets(_)
        | PostFx2d::WetReflections(_) => {
            panic!("expected blur effect for this fixture")
        }
        _ => panic!("expected blur effect for this fixture"),
    }
}

#[test]
fn scene_service_updates_base_opacity() {
    let service = LayeredImageSceneService::default();
    service.queue(LayeredImageDrawCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "main-menu-background".to_owned(),
        render_layer: "background.city".to_owned(),
        image: LayeredImageInstance {
            asset: AssetKey::new("test-mod/layered-images/test-scene"),
            size: Vec2::new(1280.0, 720.0),
            base_opacity: 1.0,
            viewport_fit: LayeredImageViewportFit2d::Fixed,
            visual_maps: None,
            layer_overrides: Vec::new(),
        },
        z_index: -100.0,
        transform: Transform2::default(),
    });

    assert!(service.set_base_opacity("main-menu-background", 0.35));
    assert!(!service.set_base_opacity("missing-background", 0.35));

    let command = service.commands().remove(0);
    assert_eq!(command.image.base_opacity, 0.35);
}

#[test]
fn can_handle_layered_image_scene_command_returns_true_for_layered_image_command() {
    let command = SceneCommand::QueueLayeredImage2d {
        command: amigo_scene::LayeredImage2dSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "background".to_owned(),
            asset: AssetKey::new("test-mod/layered-images/test-scene"),
            size: Vec2::new(1280.0, 720.0),
            base_opacity: 1.0,
            viewport_fit: amigo_scene::LayeredImageViewportFit2dSceneCommand::Fixed,
            visual_maps: None,
            layer_overrides: Vec::new(),
            render_layer: "background.city".to_owned(),
            z_index: -100.0,
            transform: Transform2::default(),
        },
    };

    assert!(can_handle_layered_image_scene_command(&command));
}

#[test]
fn handle_layered_image_scene_command_queues_image_and_publishes_event() {
    let scene_service = SceneService::default();
    let layered_image_scene_service = LayeredImageSceneService::default();
    let scene_event_queue = SceneEventQueue::default();
    let command = SceneCommand::QueueLayeredImage2d {
        command: amigo_scene::LayeredImage2dSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "background".to_owned(),
            asset: AssetKey::new("test-mod/layered-images/test-scene"),
            size: Vec2::new(1280.0, 720.0),
            base_opacity: 1.0,
            viewport_fit: amigo_scene::LayeredImageViewportFit2dSceneCommand::Fixed,
            visual_maps: None,
            layer_overrides: Vec::new(),
            render_layer: "background.city".to_owned(),
            z_index: -100.0,
            transform: Transform2::default(),
        },
    };

    let outcome = handle_layered_image_scene_command(
        LayeredImageSceneCommandContext {
            scene_service: &scene_service,
            layered_image_scene_service: &layered_image_scene_service,
            scene_event_queue: &scene_event_queue,
        },
        command,
    )
    .expect("layered image command should be handled");

    assert_eq!(outcome.entity_name, "background");
    assert_eq!(outcome.source_mod, "test-mod");
    assert_eq!(outcome.asset.as_str(), "test-mod/layered-images/test-scene");
    assert_eq!(scene_service.entity_names(), vec!["background".to_owned()]);
    assert_eq!(layered_image_scene_service.commands().len(), 1);

    let events = scene_event_queue.drain();
    assert_eq!(events.len(), 1);
    match &events[0] {
        SceneEvent::EntitySpawned { entity_id, name } => {
            assert_eq!(*entity_id, 0);
            assert_eq!(name, "background");
        }
        other => panic!("expected entity spawned event, got {other:?}"),
    }
}

#[test]
fn runtime_control_sets_layer_opacity() {
    let service = Arc::new(LayeredImageSceneService::default());
    let assets = Arc::new(AssetCatalog::default());
    assets.mark_prepared(test_prepared_layered_image());
    service.queue(LayeredImageDrawCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "city-bg".to_owned(),
        render_layer: "background.city".to_owned(),
        image: LayeredImageInstance {
            asset: AssetKey::new("test-mod/layered-images/test-scene"),
            size: Vec2::new(1280.0, 720.0),
            base_opacity: 1.0,
            viewport_fit: LayeredImageViewportFit2d::Fixed,
            visual_maps: None,
            layer_overrides: Vec::new(),
        },
        z_index: 0.0,
        transform: Transform2::default(),
    });

    let control = RuntimeControlService::default();
    control.register_provider(Arc::new(crate::LayeredImage2dControlProvider::new(
        service.clone(),
        assets,
    )));
    control
        .set(
            "world.background.city.LayeredImage2D.layers.windows.opacity",
            ControlValue::F64(0.35),
        )
        .expect("layer opacity should update");

    let command = service.commands().remove(0);
    assert_eq!(command.image.layer_overrides[0].id, "windows");
    assert_eq!(command.image.layer_overrides[0].opacity, Some(0.35));
}
