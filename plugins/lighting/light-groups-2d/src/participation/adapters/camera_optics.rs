use amigo_camera_optics_plugin::api::CameraOpticalCoverage2d;

use crate::api::LightGroup2dSource;

pub fn light_group_to_camera_optics_coverage(
    group: &LightGroup2dSource,
) -> CameraOpticalCoverage2d {
    match (&group.lightmap_source, &group.lightmap_channel) {
        (Some(source), Some(channel)) => CameraOpticalCoverage2d::LightMapChannel {
            source: source.clone(),
            channel: channel.clone(),
        },
        _ => CameraOpticalCoverage2d::Unsupported {
            reason: "light_group_missing_lightmap_channel".to_owned(),
        },
    }
}
