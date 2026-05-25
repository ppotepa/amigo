use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Transform3};

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
