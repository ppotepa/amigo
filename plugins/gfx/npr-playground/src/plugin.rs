use crate::render::NprPlaygroundRenderService;
use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct NprPlaygroundState {
    pub rotation_x: Mutex<f32>,
    pub rotation_y: Mutex<f32>,
}

impl NprPlaygroundState {
    pub fn tick(&self, dt: f32) {
        *self.rotation_x.lock().expect("NPR rotation mutex") += dt * 0.37;
        *self.rotation_y.lock().expect("NPR rotation mutex") += dt * 0.71;
    }
}

pub struct NprPlaygroundPlugin;

impl RuntimePlugin for NprPlaygroundPlugin {
    fn name(&self) -> &'static str {
        "amigo-npr-playground-plugin"
    }
    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(NprPlaygroundState::default())?;
        registry.register(NprPlaygroundRenderService::default())?;
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "npr_playground_update",
            |runtime| {
                let state = runtime.required::<NprPlaygroundState>()?;
                state.tick(amigo_session::simulation_delta_seconds(runtime));
                let render = runtime.required::<NprPlaygroundRenderService>()?;
                let rotation_x = *state.rotation_x.lock().expect("NPR rotation mutex");
                let rotation_y = *state.rotation_y.lock().expect("NPR rotation mutex");
                render.rebuild_cube_rotated([512, 512], 0x4E5052, rotation_x, rotation_y);
                Ok(())
            },
        );
        if let Some(ids) = registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            ids.register(crate::render::NPR_PLAYGROUND_EXTRACTOR_ID);
        }
        register_domain_plugin(
            registry,
            "amigo.gfx.npr-playground",
            &["gfx.npr@1"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        Ok(())
    }
}
