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
