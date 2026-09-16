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
    pub fn from_indexed(positions: &[[f32; 3]], indices: &[u32]) -> Result<Self, String> {
        if positions.is_empty() || indices.is_empty() || indices.len() % 3 != 0 {
            return Err("NPR geometry requires vertices and triangle indices".into());
        }
        if indices.iter().any(|index| *index as usize >= positions.len()) {
            return Err("NPR geometry index is outside the vertex buffer".into());
        }
        Ok(Self {
            vertices: positions
                .iter()
                .map(|position| NprVertex { position: Vec3::from(*position) })
                .collect(),
            triangles: indices
                .chunks_exact(3)
                .map(|triangle| [triangle[0], triangle[1], triangle[2]])
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
