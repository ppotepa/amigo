use amigo_render_npr::*;
use glam::{Vec2, Vec3};

#[test]
fn builtins_have_valid_closed_topology_and_no_degenerate_faces() {
    for geometry in [
        NprGeometry::canonical_cube(),
        NprGeometry::wedge(),
        NprGeometry::cylinder(24),
        NprGeometry::icosphere(),
    ] {
        let topology = build_topology(&geometry);
        assert!(!topology.is_empty());
        assert!(topology.iter().all(|e| e.faces[1] != u32::MAX));
        for tri in &geometry.triangles {
            let [a, b, c] = tri.map(|i| geometry.vertices[i as usize].position);
            assert!((b - a).cross(c - a).length() > 1e-5);
        }
    }
}

#[test]
fn welding_coincident_indices_removes_false_import_seam_boundaries() {
    let split = NprGeometry::from_indexed(
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        &[0, 1, 2, 3, 4, 5],
    )
    .unwrap();
    let welded = split.welded_coincident_vertices();

    assert_eq!(welded.vertices.len(), 4);
    assert_eq!(welded.triangles.len(), split.triangles.len());
    assert!(
        build_topology(&welded)
            .iter()
            .any(|edge| edge.faces[0] != u32::MAX && edge.faces[1] != u32::MAX),
        "the shared edge must no longer be interpreted as two boundaries"
    );
}

#[test]
fn nearby_welding_removes_export_jitter_without_collapsing_source_triangles() {
    let split = NprGeometry::from_indexed(
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0 + 0.000_001, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0 + 0.000_001, 0.0],
        ],
        &[0, 1, 2, 3, 4, 5],
    )
    .unwrap();
    let welded = split.welded_nearby_vertices(0.000_01);

    assert_eq!(welded.vertices.len(), 4);
    assert_eq!(welded.triangles.len(), split.triangles.len());
    assert!(welded
        .triangles
        .iter()
        .all(|triangle| triangle[0] != triangle[1] && triangle[1] != triangle[2] && triangle[0] != triangle[2]));
}

#[test]
fn coplanar_groups_merge_cube_face_triangulation_but_not_cube_edges() {
    let cube = NprGeometry::canonical_cube();
    let topology = build_topology(&cube);
    let groups = coplanar_face_groups(&cube, &topology, 0.9999);
    assert_eq!(groups.len(), cube.triangles.len());
    assert_eq!(
        groups
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        6
    );
    for face in groups.chunks_exact(2) {
        assert_eq!(face[0], face[1]);
    }
}
#[test]
fn crossing_triangle_clips_to_two_finite_perspective_triangles() {
    let camera = PerspectiveCamera::cube_default(1.0);
    let point = |x: f32, y: f32, z: f32| camera.position + Vec3::new(x, y, -z);
    let clipped = camera.clip_triangle([
        point(-0.01, 0.0, 0.01),
        point(0.1, -0.1, 1.0),
        point(0.0, 0.1, 1.0),
    ]);
    assert_eq!(clipped.len(), 2);
    for tri in clipped {
        for p in tri {
            let v = camera.project(p, Vec2::splat(512.0)).unwrap();
            assert!(v.screen.is_finite());
            assert!((0.0..=1.0).contains(&camera.normalized_depth(v.depth)));
        }
    }
    assert!(camera.normalized_depth(0.1) < camera.normalized_depth(1.0));
    assert!(camera.clip_triangle([point(0.0, 0.0, 0.01); 3]).is_empty());
}
#[test]
fn open_feature_chain_starts_at_an_endpoint_even_if_first_edge_is_interior() {
    let features = [(1, 2), (0, 1), (2, 3)].map(|(a, b)| FeatureSegment {
        edge: TopologyEdge {
            a,
            b,
            faces: [0, u32::MAX],
        },
        class: FeatureClass::Boundary,
        midpoint: Vec3::ZERO,
    });
    let chains = stroke::chain_features(&features);
    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0].vertices, vec![0, 1, 2, 3]);
}

#[test]
fn cube_rotations_keep_projection_finite_and_never_promote_face_diagonals() {
    let cube = NprGeometry::canonical_cube();
    let topology = build_topology(&cube);
    let viewport = [512, 512];
    let style = ComicInk::default();

    for step in 0..36 {
        let angle = step as f32 * std::f32::consts::TAU / 36.0;
        let geometry = cube.transformed(glam::Mat4::from_euler(
            glam::EulerRot::XYZ,
            angle * 0.61,
            angle,
            angle * 0.37,
        ));
        let camera = PerspectiveCamera::cube_default(1.0);
        let packet = build_packet_with_topology(
            &geometry,
            &topology,
            camera,
            viewport,
            style,
            0x4e50_52,
            NprDebugView::Final,
        );

        assert!(!packet.fills.is_empty(), "rotation {step} lost all fills");
        assert!(packet.fills.iter().all(|triangle| {
            triangle
                .positions
                .iter()
                .all(|position| position.is_finite())
                && triangle.depths.iter().all(|depth| depth.is_finite())
        }));
        assert!(packet.strokes.iter().all(|stroke| {
            stroke
                .vertices
                .iter()
                .all(|vertex| vertex.position.is_finite())
                && stroke
                    .indices
                    .iter()
                    .all(|index| (*index as usize) < stroke.vertices.len())
        }));

        let features = classify_perspective_features(
            &geometry,
            &topology,
            camera.position,
            style.crease_angle,
        );
        assert!(
            features.iter().all(|feature| {
                feature.edge.faces[1] == u32::MAX
                    || face_normal(&geometry, feature.edge.faces[0])
                        .dot(face_normal(&geometry, feature.edge.faces[1]))
                        < 0.9999
            }),
            "a coplanar triangulation diagonal became a feature at rotation {step}"
        );
    }
}
