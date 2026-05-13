use amigo_3d_material::MaterialPlugin;
use amigo_3d_mesh::MeshPlugin;
use amigo_3d_text::Text3dPlugin;
use amigo_core::AmigoResult;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_session::RuntimeSession;

pub struct ThreeDRuntimeBundle;

impl PluginBundle for ThreeDRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-3d-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(MeshPlugin)?
            .with_plugin(Text3dPlugin)?
            .with_plugin(MaterialPlugin)
    }
}

pub fn register_three_d_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_3d_mesh::register_mesh3d_runtime_capabilities(session);
    amigo_3d_material::register_material3d_runtime_capabilities(session);
    amigo_3d_text::register_text3d_runtime_capabilities(session);
}

