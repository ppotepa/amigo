use super::*;
use amigo_2d_motion::{
    CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL, motion_runtime_plugin_report_label,
};

pub(crate) struct RuntimeDiagnosticsPlugin {
    script_backend: String,
    plugin_names: Vec<String>,
}

fn diagnostics_plugin_label(plugin_name: &str) -> String {
    motion_runtime_plugin_report_label(plugin_name)
}

impl RuntimeDiagnosticsPlugin {
    pub(crate) fn phase1() -> Self {
        Self {
            script_backend: "rhai".to_owned(),
            plugin_names: vec![
                "amigo-assets",
                "amigo-scene",
                "amigo-window-winit",
                "amigo-input-winit",
                "amigo-render-wgpu",
                "amigo-app-launch-selection",
                "amigo-app-runtime-systems",
                "amigo-app-scene-command-registry",
                "amigo-app-script-command-registry",
                "amigo-2d-sprite",
                "amigo-2d-text",
                "amigo-2d-vector",
                "amigo-ui",
                "amigo-2d-physics",
                "amigo-2d-tilemap",
                CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL,
                "amigo-audio-api",
                "amigo-audio-generated",
                "amigo-audio-mixer",
                "amigo-audio-output",
                "amigo-3d-mesh",
                "amigo-3d-text",
                "amigo-3d-material",
                "amigo-modding",
                "amigo-app-runtime-diagnostics",
                "amigo-scripting-rhai",
            ]
            .into_iter()
            .map(diagnostics_plugin_label)
            .collect(),
        }
    }
}

impl RuntimePlugin for RuntimeDiagnosticsPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-runtime-diagnostics"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let window = required_from_registry::<WindowServiceInfo>(registry)?;
        let input = required_from_registry::<InputServiceInfo>(registry)?;
        let render = required_from_registry::<RenderBackendInfo>(registry)?;

        let mut capabilities = Vec::new();
        capabilities.push(
            required_from_registry::<SpriteDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<Text2dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<VectorDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<UiDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<Physics2dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<TileMap2dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<amigo_2d_motion::Motion2dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<AudioDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<GeneratedAudioDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<AudioMixerDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<AudioOutputDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<MeshDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<Text3dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<MaterialDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.sort();

        let loaded_mods = registry
            .resolve::<ModCatalog>()
            .map(|catalog| {
                catalog
                    .mod_ids()
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut service_names = registry
            .registered_names()
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        service_names.push(type_name::<ScriptCommandQueue>().to_owned());
        service_names.push(type_name::<ScriptEventQueue>().to_owned());
        service_names.push(type_name::<DevConsoleQueue>().to_owned());
        service_names.push(type_name::<DevConsoleState>().to_owned());
        service_names.push(type_name::<ScriptRuntimeInfo>().to_owned());
        service_names.push(type_name::<ScriptRuntimeService>().to_owned());
        service_names.push(type_name::<RuntimeDiagnostics>().to_owned());
        service_names.sort();
        service_names.dedup();

        registry.register(RuntimeDiagnostics::new(
            window.backend_name,
            input.backend_name,
            render.backend_name,
            self.script_backend.clone(),
            loaded_mods,
            capabilities,
            self.plugin_names.clone(),
            service_names,
        ))
    }
}
