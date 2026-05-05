use crate::{
    AssetCatalog, AssetEvent, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest,
    AssetSourceKind,
};

#[test]
fn registers_manifest_and_requests_load() {
    let catalog = AssetCatalog::default();
    let key = AssetKey::new("core/images/logo");

    let inserted = catalog.register_manifest(AssetManifest {
        key: key.clone(),
        source: AssetSourceKind::Mod("core".to_owned()),
        tags: vec!["ui".to_owned(), "logo".to_owned()],
    });
    catalog.request_load(AssetLoadRequest::new(
        key.clone(),
        AssetLoadPriority::Immediate,
    ));

    assert!(inserted);
    assert!(catalog.contains(&key));
    assert_eq!(
        catalog.manifest(&key).expect("manifest should exist").tags,
        vec!["ui".to_owned(), "logo".to_owned()]
    );
    assert_eq!(catalog.pending_loads().len(), 1);

    let events = catalog.drain_events();
    assert_eq!(
        events,
        vec![
            AssetEvent::ManifestRegistered(key.clone()),
            AssetEvent::LoadRequested(AssetLoadRequest::new(
                key.clone(),
                AssetLoadPriority::Immediate,
            )),
        ]
    );
    assert_eq!(catalog.drain_pending_loads().len(), 1);
}

#[test]
fn coalesces_pending_loads_to_highest_priority() {
    let catalog = AssetCatalog::default();
    let key = AssetKey::new("mods/playground-3d/meshes/probe.mesh");

    catalog.request_load(AssetLoadRequest::new(
        key.clone(),
        AssetLoadPriority::Background,
    ));
    catalog.request_load(AssetLoadRequest::new(
        key.clone(),
        AssetLoadPriority::Immediate,
    ));
    catalog.request_load(AssetLoadRequest::new(
        key.clone(),
        AssetLoadPriority::Interactive,
    ));

    assert_eq!(
        catalog.pending_loads(),
        vec![AssetLoadRequest::new(key, AssetLoadPriority::Immediate)]
    );
}

#[test]
fn filters_manifests_by_mod_and_tag() {
    let catalog = AssetCatalog::default();
    let sprite_key = AssetKey::new("playground-2d/spritesheets/sprite-lab");
    let mesh_key = AssetKey::new("playground-3d/meshes/probe");

    catalog.register_manifest(AssetManifest {
        key: sprite_key.clone(),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        tags: vec!["phase3".to_owned(), "2d".to_owned(), "sprite".to_owned()],
    });
    catalog.register_manifest(AssetManifest {
        key: mesh_key.clone(),
        source: AssetSourceKind::Mod("playground-3d".to_owned()),
        tags: vec!["phase3".to_owned(), "3d".to_owned(), "mesh".to_owned()],
    });

    assert_eq!(catalog.manifests_for_mod("playground-2d").len(), 1);
    assert_eq!(catalog.manifests_with_tag("mesh").len(), 1);
    assert_eq!(
        catalog.tags_for(&mesh_key),
        vec!["phase3".to_owned(), "3d".to_owned(), "mesh".to_owned()]
    );
    assert_eq!(
        catalog
            .manifest(&sprite_key)
            .expect("sprite manifest should exist")
            .source
            .label(),
        "mod:playground-2d"
    );
}
