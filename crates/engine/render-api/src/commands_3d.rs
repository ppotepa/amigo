use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Transform3, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera3dRenderSettings {
    pub fov_y_degrees: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Default for Camera3dRenderSettings {
    fn default() -> Self {
        Self {
            fov_y_degrees: 55.0,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light3dRenderSettings {
    pub direction: Vec3,
    pub color: ColorRgba,
    pub intensity: f32,
    pub ambient: f32,
}

impl Default for Light3dRenderSettings {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.35, -0.8, -0.45),
            color: ColorRgba::WHITE,
            intensity: 0.85,
            ambient: 0.25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh3d {
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
}

#[derive(Debug, Clone)]
pub struct MeshDrawCommand {
    pub entity_id: u64,
    pub entity_name: String,
    pub mesh: Mesh3d,
}

#[derive(Debug, Clone)]
pub struct Material3d {
    pub label: String,
    pub albedo: ColorRgba,
    pub source: Option<AssetKey>,
    pub render_order: i32,
}

#[derive(Debug, Clone)]
pub struct MaterialDrawCommand {
    pub entity_id: u64,
    pub entity_name: String,
    pub material: Material3d,
}

#[derive(Debug, Clone)]
pub struct Text3d {
    pub content: String,
    pub font: AssetKey,
    pub size: f32,
    pub transform: Transform3,
}

#[derive(Debug, Clone)]
pub struct Text3dDrawCommand {
    pub entity_id: u64,
    pub entity_name: String,
    pub text: Text3d,
}

pub trait Mesh3dRenderOutput {
    fn push_mesh3d_render_command(&mut self, command: MeshDrawCommand);
}

pub trait Material3dRenderOutput {
    fn push_material3d_render_command(&mut self, command: MaterialDrawCommand);
}

pub trait Text3dRenderOutput {
    fn push_text3d_render_command(&mut self, command: Text3dDrawCommand);
}
