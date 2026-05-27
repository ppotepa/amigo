#[test]
fn tilemap_2d_waterfall_contract_exists() {
    assert_eq!(
        amigo_tilemap_2d_plugin::scene::tilemap_2d_scene_descriptor()
            .id
            .as_str(),
        "amigo.gfx.tilemap-2d.TileMap2D"
    );
}
