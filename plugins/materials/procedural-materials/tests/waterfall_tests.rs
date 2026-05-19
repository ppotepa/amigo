#[test]
fn procedural_materials_waterfall_contract_exists() {
    assert_eq!(
        amigo_procedural_materials_plugin::plugin::PLUGIN_ID,
        "amigo.materials.procedural-materials"
    );
}
