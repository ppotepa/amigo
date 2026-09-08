use glam::Vec3;

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

impl Default for NprGeometry {
    fn default() -> Self {
        Self::canonical_cube()
    }
}
