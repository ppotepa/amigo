#[test]
fn relight_2d_waterfall_contract_exists() {
    assert_eq!(
        amigo_relight_2d_plugin::plugin::PLUGIN_ID,
        "amigo.lighting.relight-2d"
    );
}
