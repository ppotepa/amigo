#[test]
fn procedural_materials_waterfall_contract_exists() {
    assert_eq!(
        amigo_procedural_materials_plugin::plugin::PLUGIN_ID,
        "amigo.materials.procedural-materials"
    );
}

#[test]
fn procedural_material_declares_generator_and_target() {
    let material = amigo_procedural_materials_plugin::ProceduralMaterial2d::new(
        "neon-noise",
        "thresholded-luma",
        amigo_procedural_materials_plugin::ProceduralMaterialTarget2d::SceneEmissive,
        7,
    );

    assert!(material.is_valid());
    assert_eq!(material.target.target_id(), "SceneEmissive");
}
