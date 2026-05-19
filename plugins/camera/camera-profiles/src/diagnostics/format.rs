use crate::api::CameraProfile2d;

pub fn format_camera_profile_2d(profile: &CameraProfile2d) -> String {
    format!(
        "camera.profile id={} label={} lens={} film={} focus_distance_m={}",
        profile.id,
        profile.label,
        profile.lens_profile.as_deref().unwrap_or("none"),
        profile.film_profile.as_deref().unwrap_or("none"),
        profile
            .focus_distance_m
            .map(|value| format!("{value:.3}"))
            .unwrap_or_else(|| "none".to_owned())
    )
}
