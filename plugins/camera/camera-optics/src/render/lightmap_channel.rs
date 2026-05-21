use crate::api::CameraOpticalCoverage2d;

pub fn lightmap_channel_parts(coverage: &CameraOpticalCoverage2d) -> Option<(&str, &str)> {
    match coverage {
        CameraOpticalCoverage2d::LightMapChannel { source, channel } => {
            Some((source.as_str(), channel.as_str()))
        }
        _ => None,
    }
}
