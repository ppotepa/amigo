use amigo_camera_optics_plugin::api::{
    CameraOpticalEmitterKind2d, CameraOpticalResponse2d, CameraOpticalSource2d,
    CameraOpticalSourceStatus2d,
};

use crate::api::BeaconLight2dSource;

pub fn beacon_to_camera_optics_source(beacon: &BeaconLight2dSource) -> CameraOpticalSource2d {
    CameraOpticalSource2d {
        owner: beacon.id.clone(),
        component_kind: "BeaconLight2D".to_owned(),
        emitter_kind: CameraOpticalEmitterKind2d::Beacon,
        source_id: None,
        render_layer: None,
        color_rgba: Some(beacon.color_rgba),
        intensity: Some(beacon.intensity),
        effective_intensity: Some(beacon.intensity),
        response: CameraOpticalResponse2d {
            enabled: true,
            intensity: beacon.intensity,
            bloom: beacon.intensity,
            glare: beacon.intensity,
            ..CameraOpticalResponse2d::default()
        },
        status: CameraOpticalSourceStatus2d::Active,
        reason: "beacon_camera_optics_adapter".to_owned(),
        position_px: None,
        radius_px: Some(beacon.radius_px),
        roles: amigo_plugin_api::RenderContributionSet::from_pairs([
            (amigo_plugin_api::roles::CAMERA_FX_SOURCE, true),
            (amigo_plugin_api::roles::BLOOM_SOURCE, true),
        ]),
    }
}
