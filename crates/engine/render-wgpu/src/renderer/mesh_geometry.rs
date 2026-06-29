use std::collections::BTreeMap;
use std::sync::Arc;

use amigo_math::Vec3;

use crate::renderer::{WgpuSceneRenderer, cross, normalize, sub};

const NPR_EDGE_WELD_EPSILON: f32 = 1.0e-5;

#[derive(Debug, Clone)]
pub(crate) struct CachedMeshGeometry3d {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) triangles: Vec<MeshTriangle3d>,
    pub(crate) edges: Vec<MeshEdge3d>,
}

impl CachedMeshGeometry3d {
    #[cfg(test)]
    pub(crate) fn from_test_vertices(vertices: Vec<Vec3>) -> Self {
        Self {
            vertices,
            triangles: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub(crate) fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    pub(crate) fn triangles(&self) -> &[MeshTriangle3d] {
        &self.triangles
    }

    pub(crate) fn edges(&self) -> &[MeshEdge3d] {
        &self.edges
    }

    pub(crate) fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub(crate) fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub(crate) fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MeshTriangle3d {
    pub(crate) indices: [usize; 3],
    pub(crate) normal: Vec3,
    pub(crate) material_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct MeshEdge3d {
    pub(crate) edge_id: u64,
    pub(crate) a: usize,
    pub(crate) b: usize,
    pub(crate) faces: Vec<usize>,
    pub(crate) material_seam: bool,
}

pub(crate) fn mesh_geometry_from_asset(
    assets: &dyn amigo_render_api::RenderAssetSource,
    mesh_asset: &amigo_assets::AssetKey,
) -> Option<CachedMeshGeometry3d> {
    let prepared = assets.prepared_asset(mesh_asset)?;
    if !matches!(prepared.kind, amigo_assets::PreparedAssetKind::Mesh3d) {
        return None;
    }
    let source_file = prepared.metadata.get("source.file")?;
    let mesh_path = prepared.resolved_path.parent()?.join(source_file);
    if prepared.format.as_deref() != Some("glb") && mesh_path.extension()?.to_str()? != "glb" {
        return None;
    }
    load_glb_geometry(&mesh_path).ok().filter(|geometry| {
        !geometry.vertices.is_empty()
            && !geometry.triangles.is_empty()
            && !geometry.edges.is_empty()
    })
}

impl WgpuSceneRenderer {
    pub(crate) fn mesh_geometry_3d(
        &mut self,
        assets: &dyn amigo_render_api::RenderAssetSource,
        mesh_asset: &amigo_assets::AssetKey,
    ) -> Arc<CachedMeshGeometry3d> {
        let cache_key = mesh_asset.as_str().to_owned();
        if let Some(cached) = self.mesh_3d_geometry_cache.get(&cache_key) {
            return Arc::clone(cached);
        }

        let geometry =
            Arc::new(mesh_geometry_from_asset(assets, mesh_asset).unwrap_or_else(cube_geometry));
        self.mesh_3d_geometry_cache
            .insert(cache_key, Arc::clone(&geometry));
        geometry
    }
}

pub(crate) fn load_glb_geometry(
    path: &std::path::Path,
) -> Result<CachedMeshGeometry3d, gltf::Error> {
    let (document, buffers, _) = gltf::import(path)?;
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    if let Some(scene) = document
        .default_scene()
        .or_else(|| document.scenes().next())
    {
        for node in scene.nodes() {
            append_gltf_node_geometry(
                &mut vertices,
                &mut triangles,
                &buffers,
                node,
                GltfNodeTransform3d::IDENTITY,
            );
        }
    }

    if vertices.is_empty() {
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                append_gltf_primitive_geometry(
                    &mut vertices,
                    &mut triangles,
                    &buffers,
                    primitive,
                    GltfNodeTransform3d::IDENTITY,
                );
            }
        }
    }

    normalize_geometry(&mut vertices);
    rebuild_triangle_normals(&mut triangles, &vertices);
    let edges = build_edges(&vertices, &triangles);
    Ok(CachedMeshGeometry3d {
        vertices,
        triangles,
        edges,
    })
}

fn append_gltf_node_geometry(
    vertices: &mut Vec<Vec3>,
    triangles: &mut Vec<MeshTriangle3d>,
    buffers: &[gltf::buffer::Data],
    node: gltf::Node<'_>,
    parent_transform: GltfNodeTransform3d,
) {
    let node_transform = parent_transform.then(GltfNodeTransform3d::from_node(&node));
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            append_gltf_primitive_geometry(vertices, triangles, buffers, primitive, node_transform);
        }
    }
    for child in node.children() {
        append_gltf_node_geometry(vertices, triangles, buffers, child, node_transform);
    }
}

fn append_gltf_primitive_geometry(
    vertices: &mut Vec<Vec3>,
    triangles: &mut Vec<MeshTriangle3d>,
    buffers: &[gltf::buffer::Data],
    primitive: gltf::Primitive<'_>,
    transform: GltfNodeTransform3d,
) {
    let material_id = primitive.material().index().map(|index| index as u32);
    let reader =
        primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| data.0.as_slice()));
    let Some(positions) = reader.read_positions() else {
        return;
    };
    let base_index = vertices.len();
    vertices.extend(positions.map(|position| {
        transform.transform_point(Vec3::new(position[0], position[1], position[2]))
    }));
    if let Some(indices) = reader.read_indices() {
        let indices = indices.into_u32().collect::<Vec<_>>();
        for chunk in indices.chunks_exact(3) {
            push_imported_triangle(
                triangles,
                vertices,
                [
                    base_index + chunk[0] as usize,
                    base_index + chunk[1] as usize,
                    base_index + chunk[2] as usize,
                ],
                material_id,
            );
        }
    } else {
        let count = vertices.len() - base_index;
        for chunk_start in (0..count).step_by(3) {
            if chunk_start + 2 >= count {
                break;
            }
            push_imported_triangle(
                triangles,
                vertices,
                [
                    base_index + chunk_start,
                    base_index + chunk_start + 1,
                    base_index + chunk_start + 2,
                ],
                material_id,
            );
        }
    }
}

#[derive(Clone, Copy)]
struct GltfNodeTransform3d {
    matrix: [f32; 16],
}

impl GltfNodeTransform3d {
    const IDENTITY: Self = Self {
        matrix: [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    fn from_node(node: &gltf::Node<'_>) -> Self {
        let (translation, rotation, scale) = node.transform().decomposed();
        let [mut x, mut y, mut z, mut w] = rotation;
        let length = (x * x + y * y + z * z + w * w).sqrt();
        if length > f32::EPSILON {
            x /= length;
            y /= length;
            z /= length;
            w /= length;
        } else {
            x = 0.0;
            y = 0.0;
            z = 0.0;
            w = 1.0;
        }

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;
        let [sx, sy, sz] = scale;

        Self {
            matrix: [
                (1.0 - 2.0 * (yy + zz)) * sx,
                (2.0 * (xy - wz)) * sy,
                (2.0 * (xz + wy)) * sz,
                translation[0],
                (2.0 * (xy + wz)) * sx,
                (1.0 - 2.0 * (xx + zz)) * sy,
                (2.0 * (yz - wx)) * sz,
                translation[1],
                (2.0 * (xz - wy)) * sx,
                (2.0 * (yz + wx)) * sy,
                (1.0 - 2.0 * (xx + yy)) * sz,
                translation[2],
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    fn then(self, child: Self) -> Self {
        let mut out = [0.0; 16];
        for row in 0..4 {
            for col in 0..4 {
                out[row * 4 + col] = self.matrix[row * 4] * child.matrix[col]
                    + self.matrix[row * 4 + 1] * child.matrix[4 + col]
                    + self.matrix[row * 4 + 2] * child.matrix[8 + col]
                    + self.matrix[row * 4 + 3] * child.matrix[12 + col];
            }
        }
        Self { matrix: out }
    }

    fn transform_point(self, point: Vec3) -> Vec3 {
        Vec3::new(
            self.matrix[0] * point.x
                + self.matrix[1] * point.y
                + self.matrix[2] * point.z
                + self.matrix[3],
            self.matrix[4] * point.x
                + self.matrix[5] * point.y
                + self.matrix[6] * point.z
                + self.matrix[7],
            self.matrix[8] * point.x
                + self.matrix[9] * point.y
                + self.matrix[10] * point.z
                + self.matrix[11],
        )
    }
}

fn push_imported_triangle(
    triangles: &mut Vec<MeshTriangle3d>,
    vertices: &[Vec3],
    indices: [usize; 3],
    material_id: Option<u32>,
) {
    let normal = normalize(cross(
        sub(vertices[indices[1]], vertices[indices[0]]),
        sub(vertices[indices[2]], vertices[indices[0]]),
    ));
    if normal == Vec3::ZERO {
        return;
    }
    triangles.push(MeshTriangle3d {
        indices,
        normal,
        material_id,
    });
}

fn normalize_geometry(vertices: &mut [Vec3]) {
    if vertices.is_empty() {
        return;
    }
    let mut min = vertices[0];
    let mut max = vertices[0];
    for vertex in vertices.iter().copied() {
        min.x = min.x.min(vertex.x);
        min.y = min.y.min(vertex.y);
        min.z = min.z.min(vertex.z);
        max.x = max.x.max(vertex.x);
        max.y = max.y.max(vertex.y);
        max.z = max.z.max(vertex.z);
    }
    let center = Vec3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
    let scale = if extent <= f32::EPSILON {
        1.0
    } else {
        1.8 / extent
    };
    for vertex in vertices {
        *vertex = Vec3::new(
            (vertex.x - center.x) * scale,
            (vertex.y - center.y) * scale,
            (vertex.z - center.z) * scale,
        );
    }
}

fn rebuild_triangle_normals(triangles: &mut [MeshTriangle3d], vertices: &[Vec3]) {
    for triangle in triangles {
        triangle.normal = normalize(cross(
            sub(vertices[triangle.indices[1]], vertices[triangle.indices[0]]),
            sub(vertices[triangle.indices[2]], vertices[triangle.indices[0]]),
        ));
    }
}

type WeldedVertexKey3d = (i64, i64, i64);
type WeldedEdgeKey3d = (WeldedVertexKey3d, WeldedVertexKey3d);

#[derive(Debug, Clone)]
struct MeshEdgeAccumulator3d {
    a: usize,
    b: usize,
    faces: Vec<usize>,
}

pub(crate) fn build_edges(vertices: &[Vec3], triangles: &[MeshTriangle3d]) -> Vec<MeshEdge3d> {
    let mut edge_faces = BTreeMap::<WeldedEdgeKey3d, MeshEdgeAccumulator3d>::new();
    for (face_index, triangle) in triangles.iter().enumerate() {
        let [a, b, c] = triangle.indices;
        for (left, right) in [(a, b), (b, c), (c, a)] {
            let left_key = welded_vertex_key(vertices[left]);
            let right_key = welded_vertex_key(vertices[right]);
            let key = if left_key <= right_key {
                (left_key, right_key)
            } else {
                (right_key, left_key)
            };
            edge_faces
                .entry(key)
                .or_insert_with(|| {
                    let (a, b) = if left <= right {
                        (left, right)
                    } else {
                        (right, left)
                    };
                    MeshEdgeAccumulator3d {
                        a,
                        b,
                        faces: Vec::new(),
                    }
                })
                .faces
                .push(face_index);
        }
    }
    edge_faces
        .into_iter()
        .map(|(_, edge)| {
            let faces = edge.faces;
            let material_seam = faces.len() == 2
                && triangles[faces[0]].material_id != triangles[faces[1]].material_id;
            MeshEdge3d {
                edge_id: stable_mesh_edge_id(edge.a, edge.b, &faces),
                a: edge.a,
                b: edge.b,
                faces,
                material_seam,
            }
        })
        .collect()
}

pub(crate) fn welded_vertex_key(position: Vec3) -> WeldedVertexKey3d {
    (
        (position.x / NPR_EDGE_WELD_EPSILON).round() as i64,
        (position.y / NPR_EDGE_WELD_EPSILON).round() as i64,
        (position.z / NPR_EDGE_WELD_EPSILON).round() as i64,
    )
}

pub(crate) fn cube_geometry() -> CachedMeshGeometry3d {
    let vertices = vec![
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ];
    let face_triangles = [
        [0usize, 2usize, 1usize],
        [0usize, 3usize, 2usize],
        [4usize, 5usize, 6usize],
        [4usize, 6usize, 7usize],
        [0usize, 1usize, 5usize],
        [0usize, 5usize, 4usize],
        [2usize, 3usize, 7usize],
        [2usize, 7usize, 6usize],
        [1usize, 2usize, 6usize],
        [1usize, 6usize, 5usize],
        [3usize, 0usize, 4usize],
        [3usize, 4usize, 7usize],
    ];
    let mut triangles = Vec::new();
    for indices in face_triangles {
        push_imported_triangle(&mut triangles, &vertices, indices, None);
    }
    let edges = build_edges(&vertices, &triangles);
    CachedMeshGeometry3d {
        vertices,
        triangles,
        edges,
    }
}

fn stable_mesh_edge_id(a: usize, b: usize, faces: &[usize]) -> u64 {
    let first_face = faces.first().copied().unwrap_or_default() as u64;
    let second_face = faces.get(1).copied().unwrap_or_default() as u64;
    ((a as u64) << 32)
        ^ (b as u64)
        ^ first_face.wrapping_mul(0x9E37_79B9)
        ^ second_face.wrapping_mul(0x85EB_CA77)
}
