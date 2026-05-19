#[test]
fn material_maps_waterfall_contract_exists() {
    assert_eq!(
        amigo_material_maps_plugin::plugin::PLUGIN_ID,
        "amigo.materials.material-maps"
    );
}
