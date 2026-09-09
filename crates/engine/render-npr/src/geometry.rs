use glam::Vec3;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprVertex {
    pub position: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprGeometry {
    pub vertices: Vec<NprVertex>,
    pub triangles: Vec<[u32; 3]>,
}

impl NprGeometry {
    pub fn wedge() -> Self {
        let positions = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
        ];
        Self::outward(
            positions,
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 4, 5],
                [0, 5, 1],
                [0, 3, 4],
                [1, 5, 2],
                [3, 2, 5],
                [3, 5, 4],
            ],
        )
    }
    pub fn cylinder(segments: u32) -> Self {
        let segments = segments.max(3);
        let mut points = vec![Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
        for i in 0..segments {
            let a = i as f32 * std::f32::consts::TAU / segments as f32;
            points.extend([
                Vec3::new(a.cos(), -1.0, a.sin()),
                Vec3::new(a.cos(), 1.0, a.sin()),
            ]);
        }
        let mut triangles = Vec::new();
        for i in 0..segments {
            let a = 2 + i * 2;
            let b = 2 + ((i + 1) % segments) * 2;
            triangles.extend([
                [0, a, b],
                [1, b + 1, a + 1],
                [a, a + 1, b],
                [b, a + 1, b + 1],
            ]);
        }
        Self::outward(points, triangles)
    }
    pub fn icosphere() -> Self {
        let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let p = vec![
            Vec3::new(-1.0, t, 0.0),
            Vec3::new(1.0, t, 0.0),
            Vec3::new(-1.0, -t, 0.0),
            Vec3::new(1.0, -t, 0.0),
            Vec3::new(0.0, -1.0, t),
            Vec3::new(0.0, 1.0, t),
            Vec3::new(0.0, -1.0, -t),
            Vec3::new(0.0, 1.0, -t),
            Vec3::new(t, 0.0, -1.0),
            Vec3::new(t, 0.0, 1.0),
            Vec3::new(-t, 0.0, -1.0),
            Vec3::new(-t, 0.0, 1.0),
        ]
        .into_iter()
        .map(Vec3::normalize)
        .collect::<Vec<_>>();
        let mut triangles = Vec::new();
        for a in 0..12 {
            for b in a + 1..12 {
                for c in b + 1..12 {
                    let n = (p[b] - p[a]).cross(p[c] - p[a]);
                    let signs = p.iter().map(|v| n.dot(*v - p[a])).collect::<Vec<_>>();
                    if signs.iter().all(|s| *s >= -1e-5) || signs.iter().all(|s| *s <= 1e-5) {
                        triangles.push([a as u32, b as u32, c as u32]);
                    }
                }
            }
        }
        Self::outward(p, triangles)
    }
    fn outward(positions: Vec<Vec3>, mut triangles: Vec<[u32; 3]>) -> Self {
        let center = positions.iter().copied().sum::<Vec3>() / positions.len() as f32;
        for tri in &mut triangles {
            let [a, b, c] = tri.map(|i| positions[i as usize]);
            if (b - a).cross(c - a).dot((a + b + c) / 3.0 - center) < 0.0 {
                tri.swap(1, 2);
            }
        }
        Self {
            vertices: positions
                .into_iter()
                .map(|position| NprVertex { position })
                .collect(),
            triangles,
        }
    }
    pub fn transformed(&self, transform: glam::Mat4) -> Self {
        Self {
            vertices: self
                .vertices
                .iter()
                .map(|v| NprVertex {
                    position: transform.transform_point3(v.position),
                })
                .collect(),
            triangles: self.triangles.clone(),
        }
    }

    /// Merges vertices that occupy exactly the same model-space position while
    /// retaining triangle order. Importers commonly split an otherwise smooth
    /// surface at UV or normal seams; treating those split indices as open
    /// boundaries makes a drawing look like a wireframe. The result is safe as
    /// a drawing topology because source-triangle indices and barycentrics are
    /// unchanged.
    pub fn welded_coincident_vertices(&self) -> Self {
        let mut vertex_for_position = BTreeMap::<[u32; 3], u32>::new();
        let mut remap = Vec::with_capacity(self.vertices.len());
        let mut vertices = Vec::<NprVertex>::with_capacity(self.vertices.len());
        for vertex in &self.vertices {
            let key = vertex.position.to_array().map(canonical_position_bits);
            let index = *vertex_for_position.entry(key).or_insert_with(|| {
                let index = vertices.len() as u32;
                vertices.push(*vertex);
                index
            });
            remap.push(index);
        }
        Self {
            vertices,
            triangles: self
                .triangles
                .iter()
                .map(|triangle| triangle.map(|index| remap[index as usize]))
                .collect(),
        }
    }

    /// Merges near-coincident importer seam vertices with a tolerance relative
    /// to the model's bounding-box diagonal. This is intentionally opt-in: a
    /// model author can retain exact polygonal topology while a smooth drawing
    /// proxy treats tiny export jitter as one continuous surface.
    ///
    /// Vertices from the same source triangle are never merged, so the proxy
    /// retains a valid source-triangle/barycentric chart for construction marks.
    pub fn welded_nearby_vertices(&self, relative_tolerance: f32) -> Self {
        if !relative_tolerance.is_finite() || relative_tolerance <= 0.0 || self.vertices.is_empty() {
            return self.welded_coincident_vertices();
        }
        let (minimum, maximum) = self.vertices.iter().fold(
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
            |(minimum, maximum), vertex| {
                (minimum.min(vertex.position), maximum.max(vertex.position))
            },
        );
        let tolerance = minimum.distance(maximum) * relative_tolerance;
        if !tolerance.is_finite() || tolerance <= f32::EPSILON {
            return self.welded_coincident_vertices();
        }
        let faces_by_vertex = incident_faces(self);
        let mut cells = BTreeMap::<[i64; 3], Vec<u32>>::new();
        let mut vertices = Vec::<NprVertex>::with_capacity(self.vertices.len());
        let mut members = Vec::<Vec<usize>>::with_capacity(self.vertices.len());
        let mut remap = Vec::with_capacity(self.vertices.len());
        for (original, vertex) in self.vertices.iter().enumerate() {
            let cell = spatial_cell(vertex.position, tolerance);
            let mut selected = None;
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let neighbor = [cell[0] + x, cell[1] + y, cell[2] + z];
                        for &candidate in cells.get(&neighbor).into_iter().flatten() {
                            let candidate_index = candidate as usize;
                            if vertices[candidate_index].position.distance(vertex.position) <= tolerance
                                && !shares_source_face(
                                    &faces_by_vertex[original],
                                    &members[candidate_index],
                                    &faces_by_vertex,
                                )
                            {
                                selected = Some(selected.map_or(candidate, |current: u32| current.min(candidate)));
                            }
                        }
                    }
                }
            }
            let index = selected.unwrap_or_else(|| {
                let index = vertices.len() as u32;
                vertices.push(*vertex);
                members.push(Vec::new());
                cells.entry(cell).or_default().push(index);
                index
            });
            members[index as usize].push(original);
            remap.push(index);
        }
        Self {
            vertices,
            triangles: self
                .triangles
                .iter()
                .map(|triangle| triangle.map(|index| remap[index as usize]))
                .collect(),
        }
    }
    pub fn from_indexed(positions: &[[f32; 3]], indices: &[u32]) -> Result<Self, String> {
        if indices.len() % 3 != 0 || positions.is_empty() {
            return Err("mesh requires triangle indices and positions".into());
        }
        if positions.iter().flatten().any(|n| !n.is_finite())
            || indices.iter().any(|i| *i as usize >= positions.len())
        {
            return Err("invalid mesh position/index".into());
        }
        Ok(Self {
            vertices: positions
                .iter()
                .map(|p| NprVertex {
                    position: Vec3::from_array(*p),
                })
                .collect(),
            triangles: indices
                .chunks_exact(3)
                .map(|t| [t[0], t[1], t[2]])
                .collect(),
        })
    }
    pub fn canonical_cube() -> Self {
        let positions = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
        ];
        let triangles = vec![
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [3, 7, 6],
            [3, 6, 2],
            [0, 4, 7],
            [0, 7, 3],
            [1, 2, 6],
            [1, 6, 5],
        ];
        Self {
            vertices: positions
                .into_iter()
                .map(|position| NprVertex { position })
                .collect(),
            triangles,
        }
    }
}

fn canonical_position_bits(value: f32) -> u32 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

fn spatial_cell(position: Vec3, size: f32) -> [i64; 3] {
    position
        .to_array()
        .map(|coordinate| (coordinate / size).floor() as i64)
}

fn incident_faces(geometry: &NprGeometry) -> Vec<Vec<usize>> {
    let mut output = vec![Vec::new(); geometry.vertices.len()];
    for (face, triangle) in geometry.triangles.iter().enumerate() {
        for &vertex in triangle {
            output[vertex as usize].push(face);
        }
    }
    output
}

fn shares_source_face(
    faces: &[usize],
    candidate_members: &[usize],
    faces_by_vertex: &[Vec<usize>],
) -> bool {
    candidate_members.iter().any(|&member| {
        let candidate_faces = &faces_by_vertex[member];
        faces.iter().any(|face| candidate_faces.contains(face))
    })
}

impl Default for NprGeometry {
    fn default() -> Self {
        Self::canonical_cube()
    }
}
