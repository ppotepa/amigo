use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::{
    AssetCatalog, AssetEvent, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest,
    AssetSourceKind, LoadedAsset, PreparedAsset, PreparedAssetKind,
};

#[test]
fn tracks_loaded_and_failed_asset_states() {
    let catalog = AssetCatalog::default();
    let loaded_key = AssetKey::new("playground-2d/spritesheets/sprite-lab");
    let failed_key = AssetKey::new("playground-3d/materials/missing");

    catalog.register_manifest(AssetManifest {
        key: loaded_key.clone(),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        tags: vec!["2d".to_owned()],
    });
    catalog.register_manifest(AssetManifest {
        key: failed_key.clone(),
        source: AssetSourceKind::Mod("playground-3d".to_owned()),
        tags: vec!["3d".to_owned()],
    });
    catalog.request_load(AssetLoadRequest::new(
        loaded_key.clone(),
        AssetLoadPriority::Immediate,
    ));
    catalog.request_load(AssetLoadRequest::new(
        failed_key.clone(),
        AssetLoadPriority::Interactive,
    ));

    catalog.mark_loaded(LoadedAsset {
        key: loaded_key.clone(),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        resolved_path: PathBuf::from("mods/playground-2d/spritesheets/sprite-lab/spritesheet.yml"),
        byte_len: 42,
    });
    catalog.mark_failed(failed_key.clone(), "file not found");

    assert!(catalog.is_loaded(&loaded_key));
    assert!(catalog.is_failed(&failed_key));
    assert_eq!(catalog.pending_loads().len(), 0);
    assert_eq!(
        catalog
            .loaded_asset(&loaded_key)
            .expect("loaded asset should exist")
            .byte_len,
        42
    );
    assert_eq!(
        catalog
            .failed_asset(&failed_key)
            .expect("failed asset should exist")
            .reason,
        "file not found"
    );
    assert_eq!(catalog.loaded_assets().len(), 1);
    assert_eq!(catalog.prepared_assets().len(), 0);
    assert_eq!(catalog.failed_assets().len(), 1);
}

#[test]
fn tracks_prepared_asset_states() {
    let catalog = AssetCatalog::default();
    let key = AssetKey::new("playground-2d/spritesheets/sprite-lab");

    catalog.mark_prepared(PreparedAsset {
        key: key.clone(),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        resolved_path: PathBuf::from("mods/playground-2d/spritesheets/sprite-lab/spritesheet.yml"),
        byte_len: 84,
        kind: PreparedAssetKind::Sprite2d,
        label: Some("Sprite Lab Placeholder".to_owned()),
        format: Some("debug-placeholder".to_owned()),
        metadata: BTreeMap::from([
            ("kind".to_owned(), "sprite-2d".to_owned()),
            ("label".to_owned(), "Sprite Lab Placeholder".to_owned()),
            ("format".to_owned(), "debug-placeholder".to_owned()),
        ]),
    });

    assert!(catalog.is_prepared(&key));
    assert_eq!(catalog.prepared_assets().len(), 1);
    assert_eq!(
        catalog
            .prepared_asset(&key)
            .expect("prepared asset should exist")
            .kind
            .as_str(),
        "sprite-2d"
    );
}

#[test]
fn request_reload_requeues_loaded_and_prepared_asset() {
    let catalog = AssetCatalog::default();
    let key = AssetKey::new("playground-2d/spritesheets/sprite-lab");

    catalog.mark_loaded(LoadedAsset {
        key: key.clone(),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        resolved_path: PathBuf::from("mods/playground-2d/spritesheets/sprite-lab/spritesheet.yml"),
        byte_len: 84,
    });
    catalog.mark_prepared(PreparedAsset {
        key: key.clone(),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        resolved_path: PathBuf::from("mods/playground-2d/spritesheets/sprite-lab/spritesheet.yml"),
        byte_len: 84,
        kind: PreparedAssetKind::Sprite2d,
        label: Some("Sprite Lab Placeholder".to_owned()),
        format: Some("debug-placeholder".to_owned()),
        metadata: BTreeMap::from([
            ("kind".to_owned(), "sprite-2d".to_owned()),
            ("label".to_owned(), "Sprite Lab Placeholder".to_owned()),
            ("format".to_owned(), "debug-placeholder".to_owned()),
        ]),
    });

    catalog.request_reload(AssetLoadRequest::new(
        key.clone(),
        AssetLoadPriority::Immediate,
    ));

    assert!(!catalog.is_loaded(&key));
    assert!(!catalog.is_prepared(&key));
    assert_eq!(
        catalog.pending_loads(),
        vec![AssetLoadRequest::new(
            key.clone(),
            AssetLoadPriority::Immediate
        )]
    );
    assert!(
        catalog.drain_events().iter().any(
            |event| matches!(event, AssetEvent::ReloadRequested(request) if request.key == key)
        )
    );
}
