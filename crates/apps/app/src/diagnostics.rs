use super::*;

pub(crate) struct RuntimeDiagnosticsPlugin {
    script_backend: String,
    plugin_names: Vec<String>,
}

impl RuntimeDiagnosticsPlugin {
    pub(crate) fn phase1() -> Self {
        Self {
            script_backend: "rhai".to_owned(),
            plugin_names: vec![
                "amigo-assets".to_owned(),
                "amigo-scene".to_owned(),
                "amigo-window-winit".to_owned(),
                "amigo-input-winit".to_owned(),
                "amigo-render-wgpu".to_owned(),
                "amigo-app-launch-selection".to_owned(),
                "amigo-app-scene-command-registry".to_owned(),
                "amigo-app-script-command-registry".to_owned(),
                "amigo-2d-sprite".to_owned(),
                "amigo-2d-text".to_owned(),
                "amigo-ui".to_owned(),
                "amigo-2d-physics".to_owned(),
                "amigo-2d-tilemap".to_owned(),
                "amigo-2d-platformer".to_owned(),
                "amigo-audio-api".to_owned(),
                "amigo-audio-generated".to_owned(),
                "amigo-audio-mixer".to_owned(),
                "amigo-audio-output".to_owned(),
                "amigo-3d-mesh".to_owned(),
                "amigo-3d-text".to_owned(),
                "amigo-3d-material".to_owned(),
                "amigo-modding".to_owned(),
                "amigo-app-runtime-diagnostics".to_owned(),
                "amigo-scripting-rhai".to_owned(),
            ],
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
            required_from_registry::<PlatformerDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<amigo_2d_platformer::Motion2dDomainInfo>(registry)?
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
