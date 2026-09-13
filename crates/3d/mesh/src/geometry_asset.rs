//! Geometry-only glTF assets and sampled poses. Materials/texture decoding are independent.
//! Topology and bind-pose normalization remain stable throughout animation playback.
mod animation;
mod import;
mod pose;
mod topology;
mod validation;

pub use animation::{
    MeshAnimationClip, MeshAnimationInterpolation, MeshAnimationProperty, MeshAnimationTrack,
};
pub use import::{load_gltf_geometry, load_gltf_geometry_source_space};
use pose::GeometryDefinition;

#[derive(Debug, Clone, Default)]
pub struct MeshGeometryFrame {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct MeshGeometryAsset {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub dropped_degenerate_triangles: usize,
    animations: Vec<MeshAnimationClip>,
    definition: GeometryDefinition,
}
impl MeshGeometryAsset {
    pub fn animations(&self) -> &[MeshAnimationClip] {
        &self.animations
    }

    /// File clip index, time in seconds. Samplers clamp independently; the consumer owns looping.
    pub fn sample_animation(&self, clip: usize, time: f32) -> Result<MeshGeometryFrame, String> {
        if !time.is_finite() || time < 0.0 {
            return Err("animation time must be finite and non-negative".into());
        }
        let animation = self
            .animations
            .get(clip)
            .ok_or("animation clip does not exist")?;
        self.definition.sample(Some(animation), time)
    }
}
