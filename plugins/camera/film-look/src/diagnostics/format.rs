use crate::api::FilmLookProfile2d;

pub fn format_film_look_profile_2d(profile: &FilmLookProfile2d) -> String {
    format!(
        "film_look.profile id={} label={} enabled={} grain={:.3} halation={:.3}",
        profile.id,
        profile.label,
        profile.response.enabled,
        profile.response.grain,
        profile.response.halation
    )
}
