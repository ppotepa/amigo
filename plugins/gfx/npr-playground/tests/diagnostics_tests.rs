#[test]
fn cube_has_expected_topology() {
    assert_eq!(amigo_render_npr::build_topology(&amigo_render_npr::NprGeometry::canonical_cube()).len(), 18);
}

