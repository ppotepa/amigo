use super::*;
use amigo_capabilities::{register_domain_plugin, CapabilityRegistry, DEFAULT_CAPABILITY_VERSION};
use amigo_2d_motion::motion_runtime_plugin_report_label;

pub(crate) struct RuntimeDiagnosticsPlugin {
    script_backend: String,
}

fn diagnostics_plugin_label(plugin_name: &str) -> String {
    motion_runtime_plugin_report_label(plugin_name)
}

impl RuntimeDiagnosticsPlugin {
    pub(crate) fn phase1() -> Self {
        Self {
            script_backend: "rhai".to_owned(),
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

        register_domain_plugin(
            registry,
            "amigo-app-runtime-diagnostics",
            &[],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;

        let mut capabilities = collect_capabilities_from_registry(registry);
        capabilities.sort();

        let mut plugin_names = collect_plugins_from_registry(registry);
        plugin_names.sort();
        plugin_names.dedup();

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
            plugin_names,
            service_names,
        ))
    }
}

fn collect_plugins_from_registry(registry: &ServiceRegistry) -> Vec<String> {
    let plugin_names = registry
        .resolve::<CapabilityRegistry>()
        .map(|catalog| catalog.plugin_names())
        .unwrap_or_else(|| vec!["amigo-app-runtime-diagnostics".to_owned()]);

    plugin_names
        .into_iter()
        .map(|plugin_name| diagnostics_plugin_label(&plugin_name))
        .collect()
}

fn collect_capabilities_from_registry(registry: &ServiceRegistry) -> Vec<String> {
    registry
        .resolve::<CapabilityRegistry>()
        .map(|catalog| catalog.capability_names())
        .unwrap_or_default()
}
