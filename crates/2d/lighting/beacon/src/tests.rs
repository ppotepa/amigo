#[test]
fn beacon_service_ticks() {
    let service = crate::BeaconLight2dSceneService::default();
    service.tick(0.016);
    assert!(service.commands().is_empty());
}
