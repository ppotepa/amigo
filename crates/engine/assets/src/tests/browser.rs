use crate::{
    AssetBrowserEntryState, AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest,
    AssetManifest, AssetSourceCatalogState, AssetSourceId, AssetSourceKind,
};

#[test]
fn browser_snapshot_reports_existing_async_lifecycle_without_loading_itself() {
    let catalog = AssetCatalog::default();
    let ready = AssetKey::new("npr/cube");
    let loading = AssetKey::new("npr/suzanne");
    catalog.register_manifest(AssetManifest {
        key: ready.clone(),
        source: AssetSourceKind::Mod("npr".into()),
        tags: vec!["mesh-3d".into()],
    });
    catalog.register_manifest(AssetManifest {
        key: loading.clone(),
        source: AssetSourceKind::Mod("npr".into()),
        tags: vec!["mesh-3d".into(), "organic".into()],
    });
    catalog.request_load(AssetLoadRequest::new(
        loading.clone(),
        AssetLoadPriority::Interactive,
    ));
    let source = AssetSourceId::from_kind(&AssetSourceKind::Mod("npr".into()));
    let snapshot = catalog.asset_source_snapshot(&source).unwrap();
    assert_eq!(snapshot.state, AssetSourceCatalogState::Loading);
    assert!(snapshot
        .entries
        .iter()
        .any(|entry| entry.key == ready && entry.state == AssetBrowserEntryState::Unloaded));
    assert!(snapshot
        .entries
        .iter()
        .any(|entry| entry.key == loading && entry.state == AssetBrowserEntryState::Loading));
}
#[test]
fn browser_source_uses_a_human_label_but_keeps_its_stable_id() {
    let catalog = AssetCatalog::default();
    let source = AssetSourceKind::Mod("npr-playground".into());
    catalog.register_manifest(AssetManifest {
        key: AssetKey::new("npr-playground/models/cube"),
        source: source.clone(),
        tags: vec!["mesh-3d".into()],
    });

    let descriptor = catalog.asset_sources().pop().unwrap();
    assert_eq!(descriptor.id.as_str(), "mod:npr-playground");
    assert_eq!(descriptor.label, "Mod · npr-playground");
}

#[test]
fn browser_external_loader_stays_loading_until_completion_and_can_retry() {
    let catalog = AssetCatalog::default();
    let key = AssetKey::new("model");
    let source = AssetSourceKind::Mod("test".into());
    catalog.register_manifest(AssetManifest { key: key.clone(), source: source.clone(), tags: vec![] });
    let source = AssetSourceId::from_kind(&source);
    catalog.begin_external_load(key.clone());
    assert!(catalog.drain_pending_loads().is_empty());
    assert_eq!(catalog.asset_source_snapshot(&source).unwrap().entries[0].state, AssetBrowserEntryState::Loading);
    catalog.mark_failed(key.clone(), "invalid mesh");
    assert!(catalog.external_loading_keys().is_empty());
    assert!(matches!(catalog.asset_source_snapshot(&source).unwrap().entries[0].state, AssetBrowserEntryState::Failed { .. }));
    catalog.begin_external_load(key);
    assert_eq!(catalog.asset_source_snapshot(&source).unwrap().entries[0].state, AssetBrowserEntryState::Loading);
}
