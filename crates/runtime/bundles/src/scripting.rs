use amigo_core::AmigoResult;
use amigo_modding::ModdingPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder, RuntimePlugin, ServiceRegistry};
use amigo_scripting_api::{ScriptBindingProviderDescriptor, ScriptBindingProviderRegistry};
use amigo_scripting_rhai::RhaiScriptingPlugin;
use amigo_session::RuntimeSession;

pub use amigo_scripting_rhai::tick_script_components;

pub struct ScriptingRuntimeBundle {
    pub modding_plugin: ModdingPlugin,
}

struct DefaultScriptBindingProvidersPlugin;

const PROVIDERS: &[(&str, &str, &[&str])] = &[
    ("amigo.assets", "assets", &["assets"]),
    ("amigo.audio", "audio", &["audio"]),
    ("amigo.camera.camera-core", "camera", &["camera"]),
    ("amigo.physics.2d", "physics", &["physics"]),
    ("amigo.physics.3d", "physics3d", &["physics3d"]),
    ("amigo.postfx.composite", "postfx", &["postfx"]),
    ("amigo.gameplay.pools", "pools", &["pools"]),
    ("amigo.gameplay.projectiles", "projectiles", &["projectiles"]),
    ("amigo.modding", "mod", &["mod"]),
    ("amigo.motion.2d", "motion", &["motion"]),
    ("amigo.vfx.particles-2d", "particles", &["particles"]),
    ("amigo.gfx.sprite-2d", "sprite2d", &["sprite2d"]),
    ("amigo.gfx.layered-image-2d", "layered_image2d", &["layered_image2d"]),
    ("amigo.lighting.beacon-light-2d", "beacon2d", &["beacon2d"]),
    ("amigo.lighting.light-2d", "light2d", &["light2d"]),
    ("amigo.render.2d", "render2d", &["render2d"]),
    ("amigo.state", "state", &["state"]),
    ("amigo.gfx.vector-2d", "vector2d", &["vector2d"]),
    ("amigo.gfx.text-2d", "text2d", &["text2d"]),
    ("amigo.mesh.3d", "mesh3d", &["mesh3d"]),
    ("amigo.material.3d", "material3d", &["material3d"]),
    ("amigo.text.3d", "text3d", &["text3d"]),
    ("amigo.ui", "ui", &["ui"]),
];

impl RuntimePlugin for DefaultScriptBindingProvidersPlugin {
    fn name(&self) -> &'static str {
        "amigo-script-binding-providers"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        if !registry.has::<ScriptBindingProviderRegistry>() {
            registry.register(ScriptBindingProviderRegistry::default())?;
        }
        let providers = registry.required::<ScriptBindingProviderRegistry>()?;
        for (owner, namespace, bindings) in PROVIDERS {
            let mut descriptor = ScriptBindingProviderDescriptor::new(*owner, *namespace);
            for binding in *bindings {
                descriptor = descriptor.with_binding(*binding);
            }
            providers.register(descriptor)?;
        }
        Ok(())
    }
}

impl PluginBundle for ScriptingRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-modding-and-scripting-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(self.modding_plugin)?
            .with_plugin(DefaultScriptBindingProvidersPlugin)?
            .with_plugin(RhaiScriptingPlugin)
    }
}

pub fn register_modding_and_scripting_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_scripting_rhai::register_rhai_runtime_capabilities(session);
}
