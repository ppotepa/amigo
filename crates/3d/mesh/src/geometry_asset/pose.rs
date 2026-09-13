use super::{MeshAnimationClip, MeshAnimationProperty, MeshGeometryFrame};
use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone)]
pub(super) struct Node {
    pub parent: Option<usize>,
    pub path: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub matrix: Option<Mat4>,
    pub weights: Vec<f32>,
}
#[derive(Debug, Clone)]
pub(super) struct Skin {
    pub joints: Vec<usize>,
    pub inverse_bind: Vec<Mat4>,
}
#[derive(Debug, Clone)]
pub(super) struct Primitive {
    pub node: usize,
    pub positions: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub morph_targets: Vec<Vec<Vec3>>,
    pub skin: Option<usize>,
    pub influences: Vec<Vec<(usize, f32)>>,
}
#[derive(Debug, Clone, Default)]
pub(super) struct GeometryDefinition {
    pub nodes: Vec<Node>,
    pub order: Vec<usize>,
    pub primitives: Vec<Primitive>,
    pub skins: Vec<Skin>,
    pub normalization: Mat4,
}
impl GeometryDefinition {
    pub fn sample(
        &self,
        animation: Option<&MeshAnimationClip>,
        time: f32,
    ) -> Result<MeshGeometryFrame, String> {
        let mut poses = self
            .nodes
            .iter()
            .map(|n| (n.translation, n.rotation, n.scale))
            .collect::<Vec<_>>();
        let mut weights = self
            .nodes
            .iter()
            .map(|n| n.weights.clone())
            .collect::<Vec<_>>();
        if let Some(animation) = animation {
            for track in &animation.tracks {
                let value = track.sample(time)?;
                match track.property {
                    MeshAnimationProperty::Translation => {
                        poses[track.node].0 = Vec3::new(value[0], value[1], value[2])
                    }
                    MeshAnimationProperty::Rotation => {
                        poses[track.node].1 =
                            Quat::from_xyzw(value[0], value[1], value[2], value[3])
                    }
                    MeshAnimationProperty::Scale => {
                        poses[track.node].2 = Vec3::new(value[0], value[1], value[2])
                    }
                    MeshAnimationProperty::Weights => weights[track.node] = value,
                }
            }
        }
        let mut global = vec![Mat4::IDENTITY; self.nodes.len()];
        for &index in &self.order {
            let node = &self.nodes[index];
            let (translation, rotation, scale) = poses[index];
            let local = node.matrix.unwrap_or_else(|| {
                Mat4::from_scale_rotation_translation(scale, rotation, translation)
            });
            global[index] = node.parent.map_or(Mat4::IDENTITY, |parent| global[parent]) * local;
        }
        let joint_matrices = self
            .skins
            .iter()
            .map(|skin| {
                skin.joints
                    .iter()
                    .zip(&skin.inverse_bind)
                    .map(|(joint, bind)| global[*joint] * bind)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut frame = MeshGeometryFrame::default();
        for primitive in &self.primitives {
            let offset =
                u32::try_from(frame.positions.len()).map_err(|_| "mesh has too many vertices")?;
            for (index, &base) in primitive.positions.iter().enumerate() {
                let mut point = base;
                for (target, weight) in primitive.morph_targets.iter().zip(&weights[primitive.node])
                {
                    point += target[index] * weight;
                }
                let point = if let Some(skin) = primitive.skin {
                    primitive.influences[index]
                        .iter()
                        .map(|(joint, weight)| {
                            joint_matrices[skin][*joint].transform_point3(point) * weight
                        })
                        .sum::<Vec3>()
                } else {
                    global[primitive.node].transform_point3(point)
                };
                let point = self.normalization.transform_point3(point);
                if !point.is_finite() {
                    return Err("animation produced non-finite geometry".into());
                }
                frame.positions.push(point.to_array());
            }
            let mirrored = primitive.skin.is_none() && global[primitive.node].determinant() < 0.0;
            for triangle in primitive.indices.chunks_exact(3) {
                let mut indices = [
                    triangle[0] + offset,
                    triangle[1] + offset,
                    triangle[2] + offset,
                ];
                if mirrored {
                    indices.swap(1, 2);
                }
                frame.indices.extend(indices);
            }
        }
        Ok(frame)
    }
}
