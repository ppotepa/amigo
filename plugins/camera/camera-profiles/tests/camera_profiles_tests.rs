use amigo_camera_profiles_plugin::api::CameraProfile2d;
use amigo_camera_profiles_plugin::runtime::CameraProfileRegistry2d;

#[test]
fn camera_profile_registry_returns_inserted_profile() {
    let mut registry = CameraProfileRegistry2d::default();
    let mut profile = CameraProfile2d::new("main-menu", "Main Menu");
    profile.focus_distance_m = Some(6.0);
    registry.insert(profile);

    let loaded = registry.get("main-menu").unwrap();

    assert_eq!(loaded.label, "Main Menu");
    assert_eq!(loaded.focus_distance_m, Some(6.0));
}
