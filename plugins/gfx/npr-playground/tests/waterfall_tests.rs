#[test]
fn manifest_and_service_use_typed_npr_contract() {
    let service = amigo_npr_playground_plugin::NprPlaygroundRenderService::default();
    service.rebuild_cube([512, 512], 7);
    assert_eq!(service.snapshot().unwrap().packet.stats.geometry, 1);
}

