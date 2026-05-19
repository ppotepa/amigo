#[test]
fn film_look_plugin_owns_scene_descriptor() {
    let descriptor = amigo_film_look_plugin::scene::film_look_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.camera.film-look.FilmLook");
}
