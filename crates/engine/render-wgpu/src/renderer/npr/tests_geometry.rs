use amigo_math::Vec3;

use crate::renderer::*;

#[test]
fn cube_geometry_exposes_edges_for_npr_lines() {
    let geometry = cube_geometry();
    assert_eq!(geometry.vertices.len(), 8);
    assert_eq!(geometry.triangles.len(), 12);
    assert!(!geometry.edges.is_empty());
}

#[test]
fn npr_edge_topology_welds_duplicate_vertices_on_flat_triangle_splits() {
    let vertices = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ];
    let triangles = vec![
        MeshTriangle3d {
            indices: [0, 1, 2],
            normal: Vec3::new(0.0, 0.0, 1.0),
            material_id: Some(0),
        },
        MeshTriangle3d {
            indices: [3, 4, 5],
            normal: Vec3::new(0.0, 0.0, 1.0),
            material_id: Some(0),
        },
    ];

    let edges = build_edges(&vertices, &triangles);
    let boundary_edges = edges.iter().filter(|edge| edge.faces.len() == 1).count();
    let shared_edges = edges.iter().filter(|edge| edge.faces.len() == 2).count();

    assert_eq!(edges.len(), 5);
    assert_eq!(boundary_edges, 4);
    assert_eq!(shared_edges, 1);
    assert!(
        edges.iter().any(|edge| {
            edge.faces == vec![0, 1]
                && welded_vertex_key(vertices[edge.a]) == welded_vertex_key(vertices[1])
                && welded_vertex_key(vertices[edge.b]) == welded_vertex_key(vertices[2])
        }),
        "duplicated diagonal vertices should form one shared internal edge"
    );
}

#[test]
fn loads_playground_npr_box_glb_geometry() {
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();
    let path = workspace_root.join("mods/playground-npr/source-models/khronos/Box.glb");
    let geometry = load_glb_geometry(&path).expect("playground npr box glb should import");
    assert!(!geometry.vertices.is_empty());
    assert!(!geometry.triangles.is_empty());
    assert!(!geometry.edges.is_empty());
}

#[test]
fn loads_playground_npr_khronos_male_full_character_geometry() {
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();
    let path =
        workspace_root.join("mods/playground-npr/source-models/khronos/male/source/rigged.glb");
    let geometry = load_glb_geometry(&path).expect("playground npr Khronos male glb should import");
    let material_ids = geometry
        .triangles
        .iter()
        .filter_map(|triangle| triangle.material_id)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(
        material_ids.contains(&0),
        "body skin material should be imported"
    );
    let (body_min, body_max) =
        material_bounds(&geometry, 0).expect("body skin material should have geometry bounds");
    assert!(
        body_max.y - body_min.y > 0.75,
        "skinned body should be baked into a full-height rest pose, not collapsed near a joint"
    );
    assert!(
        (body_max.x - body_min.x).max(body_max.z - body_min.z) > 0.12,
        "skinned body should have visible width/depth after bind-pose baking"
    );
    assert!(
        material_ids.contains(&4) && material_ids.contains(&5),
        "hair materials should be imported"
    );
    assert!(
        material_ids.contains(&6) && material_ids.contains(&13),
        "face detail materials should be imported"
    );
    assert!(
        geometry.triangles.len() > 10_000,
        "full character should contain body, hair, and face triangles"
    );
}

fn material_bounds(geometry: &CachedMeshGeometry3d, material_id: u32) -> Option<(Vec3, Vec3)> {
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    let mut found = false;
    for triangle in &geometry.triangles {
        if triangle.material_id != Some(material_id) {
            continue;
        }
        found = true;
        for index in triangle.indices {
            let vertex = geometry.vertices[index];
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);
            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }
    }
    found.then_some((min, max))
}
