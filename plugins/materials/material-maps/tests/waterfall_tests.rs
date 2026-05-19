#[test]
fn material_maps_waterfall_contract_exists() {
    assert_eq!(
        amigo_material_maps_plugin::plugin::PLUGIN_ID,
        "amigo.materials.material-maps"
    );
}

#[test]
fn material_map_ref_declares_target_and_asset() {
    let map = amigo_material_maps_plugin::MaterialMapRef2d::new(
        "title",
        amigo_material_maps_plugin::MaterialMapKind2d::SceneHighlight,
        "title-highlight.png",
    );

    assert!(map.is_valid());
    assert_eq!(map.kind.target_id(), "SceneHighlight");
}
