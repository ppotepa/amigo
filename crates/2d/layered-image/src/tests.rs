use std::collections::BTreeMap;
use std::path::PathBuf;

use amigo_2d_post_fx::PostFx2d;
use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};
use amigo_math::{Transform2, Vec2};
use amigo_scene::SceneEntityId;

use crate::{
    LayeredImageBlendMode2d, LayeredImageDrawCommand, LayeredImageInstance,
    LayeredImageSceneService, LayeredImageViewportFit2d, infer_layered_image_asset_from_prepared,
};

#[test]
fn infers_layered_image_asset_from_prepared_metadata() {
    let prepared = PreparedAsset {
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
            ("layers.0.id".to_owned(), "accent_light".to_owned()),
            ("layers.0.label".to_owned(), "Accent Light".to_owned()),
            ("layers.0.image".to_owned(), "light_001.png".to_owned()),
            ("layers.0.blend".to_owned(), "additive".to_owned()),
            ("layers.0.default_opacity".to_owned(), "0.75".to_owned()),
            ("layers.0.color".to_owned(), "#FF1493".to_owned()),
            ("layers.0.post_fx.kind".to_owned(), "blur".to_owned()),
            ("layers.0.post_fx.radius".to_owned(), "18.0".to_owned()),
            ("layers.0.post_fx.downsample".to_owned(), "0.5".to_owned()),
            ("layers.0.post_fx.intensity".to_owned(), "1.2".to_owned()),
        ]),
    };

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
    let PostFx2d::Blur(blur) = post_fx.effects[0];
    assert_eq!(blur.radius, 18.0);
    assert_eq!(blur.downsample, 0.5);
    assert_eq!(blur.intensity, 1.2);
}

#[test]
fn scene_service_updates_base_opacity() {
    let service = LayeredImageSceneService::default();
    service.queue(LayeredImageDrawCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "main-menu-background".to_owned(),
        image: LayeredImageInstance {
            asset: AssetKey::new("test-mod/layered-images/test-scene"),
            size: Vec2::new(1280.0, 720.0),
            base_opacity: 1.0,
            viewport_fit: LayeredImageViewportFit2d::Fixed,
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
