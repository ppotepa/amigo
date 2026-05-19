use amigo_camera_optics_plugin::api::{
    CameraOpticalEmitterKind2d, CameraOpticalResponse2d, CameraOpticalSource2d,
    CameraOpticalSourceStatus2d,
};

use crate::api::Light2dSource;

pub fn light_to_camera_optics_source(light: &Light2dSource) -> CameraOpticalSource2d {
    CameraOpticalSource2d {
        owner: light.id.clone(),
        component_kind: "Light2D".to_owned(),
        emitter_kind: CameraOpticalEmitterKind2d::EmissiveMaterial,
        source_id: None,
        render_layer: None,
        color_rgba: Some(light.color_rgba),
        intensity: Some(light.intensity),
        effective_intensity: Some(light.intensity),
        position_px: None,
        radius_px: Some(light.radius_px),
        status: CameraOpticalSourceStatus2d::Active,
        reason: "light_2d_camera_optics_adapter".to_owned(),
        response: CameraOpticalResponse2d {
            enabled: true,
            intensity: light.intensity,
            bloom: 0.0,
            glare: light.intensity,
            ..CameraOpticalResponse2d::default()
        },
        roles: amigo_plugin_api::RenderContributionSet::from_pairs([
            (amigo_plugin_api::roles::CAMERA_FX_SOURCE, true),
        ]),
    }
}
