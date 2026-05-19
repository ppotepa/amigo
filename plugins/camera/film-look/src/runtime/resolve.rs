use crate::api::FilmLookResponse2d;

pub fn resolve_film_look_response_2d(response: FilmLookResponse2d) -> FilmLookResponse2d {
    response.normalized()
}
