use amigo_film_look_plugin::api::{FilmLookProfile2d, FilmLookResponse2d};
use amigo_film_look_plugin::runtime::resolve_film_look_response_2d;

#[test]
fn film_look_response_is_normalized() {
    let response = resolve_film_look_response_2d(FilmLookResponse2d {
        enabled: true,
        grain: 99.0,
        halation: -1.0,
        sensor_response: 2.0,
        film_response: f32::NAN,
        tone_curve: 3.0,
    });

    assert_eq!(response.grain, 4.0);
    assert_eq!(response.halation, 0.0);
    assert_eq!(response.film_response, 0.0);
}

#[test]
fn film_look_profile_has_stable_identity() {
    let profile = FilmLookProfile2d::new("rotten-noir", "Rotten Noir");

    assert_eq!(profile.id, "rotten-noir");
    assert_eq!(profile.label, "Rotten Noir");
}
