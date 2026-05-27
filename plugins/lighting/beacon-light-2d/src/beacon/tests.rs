use std::sync::Arc;

use amigo_math::{ColorRgba, Transform2};
use amigo_runtime_control::{ControlValue, RuntimeControlService};
use amigo_scene::LayeredImageViewportFit2dSceneCommand;

use super::{Beacon2dControlProvider, BeaconLight2dCommand};

#[test]
fn beacon_service_ticks() {
    let service = crate::BeaconLight2dSceneService::default();
    service.tick(0.016);
    assert!(service.commands().is_empty());
}

#[test]
fn runtime_control_sets_glow_strength() {
    let service = Arc::new(crate::BeaconLight2dSceneService::default());
    service.queue(BeaconLight2dCommand {
        source_mod: "test-mod".to_owned(),
        entity_name: "beacon-a".to_owned(),
        id: "beacon-a".to_owned(),
        render_layer: "lighting.beacon".to_owned(),
        color: ColorRgba::WHITE,
        base_intensity: 1.0,
        frequency_hz: 1.0,
        duty_cycle: 0.5,
        rise_seconds: 0.1,
        fall_seconds: 0.1,
        phase_offset: 0.0,
        sync_group: None,
        jitter_amount: 0.0,
        jitter_hz: 0.0,
        core_radius_px: 8.0,
        halo_radius_px: 16.0,
        glow_strength: 0.2,
        beam_enabled: false,
        beam_length_px: 10.0,
        beam_width_degrees: 5.0,
        beam_strength: 0.2,
        aberration_px: 0.0,

        bloom: 0.0,
        camera_response: amigo_camera_optics_plugin::api::CameraOpticalResponse2d::default(),
        distance_m: None,
        z_depth: None,
        z_index: 0.0,
        render_contributions: amigo_render_api::RenderContributionSet::default(),
        enabled: true,
        transform: Transform2::default(),
        viewport_fit: LayeredImageViewportFit2dSceneCommand::Fixed,
        viewport_canvas_size: None,
    });

    let control = RuntimeControlService::default();
    control.register_provider(Arc::new(Beacon2dControlProvider::new(service.clone())));
    control
        .set(
            "world.lighting.beacon.a.Beacon2D.glow_strength",
            ControlValue::F64(0.8),
        )
        .expect("glow strength should update");

    assert_eq!(service.commands()[0].glow_strength, 0.8);
}

#[test]
fn beacon_service_sets_z_depth() {
    let service = crate::BeaconLight2dSceneService::default();
    service.queue(BeaconLight2dCommand {
        source_mod: "test-mod".to_owned(),
        entity_name: "beacon-a".to_owned(),
        id: "beacon-a".to_owned(),
        render_layer: "lighting.beacon".to_owned(),
        color: ColorRgba::WHITE,
        base_intensity: 1.0,
        frequency_hz: 1.0,
        duty_cycle: 0.5,
        rise_seconds: 0.1,
        fall_seconds: 0.1,
        phase_offset: 0.0,
        sync_group: None,
        jitter_amount: 0.0,
        jitter_hz: 0.0,
        core_radius_px: 8.0,
        halo_radius_px: 16.0,
        glow_strength: 0.2,
        beam_enabled: false,
        beam_length_px: 10.0,
        beam_width_degrees: 5.0,
        beam_strength: 0.2,
        aberration_px: 0.0,

        bloom: 0.0,
        camera_response: amigo_camera_optics_plugin::api::CameraOpticalResponse2d::default(),
        distance_m: None,
        z_depth: None,
        z_index: 0.0,
        render_contributions: amigo_render_api::RenderContributionSet::default(),
        enabled: true,
        transform: Transform2::default(),
        viewport_fit: LayeredImageViewportFit2dSceneCommand::Fixed,
        viewport_canvas_size: None,
    });

    assert!(service.set_z_depth("beacon-a", 0.63));
    assert_eq!(service.commands()[0].z_depth, Some(0.63));
    assert!(service.set_z_depth("beacon-a", 5.0));
    assert_eq!(service.commands()[0].z_depth, Some(1.0));
    assert!(service.set_z_depth("beacon-a", -1.0));
    assert_eq!(service.commands()[0].z_depth, Some(0.0));
}
