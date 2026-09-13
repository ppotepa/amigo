use std::sync::Arc;

use amigo_core::{LaunchSelection, RuntimeDiagnostics};
use amigo_render_api::RenderFrameStatsService;

use crate::bindings::common::string_array;

#[derive(Clone)]
pub struct RuntimeApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) diagnostics: Option<Arc<RuntimeDiagnostics>>,
    pub(crate) render_stats: Option<Arc<RenderFrameStatsService>>,
}

impl RuntimeApi {
    pub fn window_backend(&mut self) -> String {
        runtime_window_backend(self.diagnostics.as_ref())
    }

    pub fn input_backend(&mut self) -> String {
        runtime_input_backend(self.diagnostics.as_ref())
    }

    pub fn render_backend(&mut self) -> String {
        runtime_render_backend(self.diagnostics.as_ref())
    }

    pub fn script_backend(&mut self) -> String {
        runtime_script_backend(self.diagnostics.as_ref())
    }

    pub fn capabilities(&mut self) -> rhai::Array {
        string_array(runtime_capabilities(self.diagnostics.as_ref()))
    }

    pub fn plugins(&mut self) -> rhai::Array {
        string_array(runtime_plugins(self.diagnostics.as_ref()))
    }

    pub fn services(&mut self) -> rhai::Array {
        string_array(runtime_services(self.diagnostics.as_ref()))
    }

    pub fn dev_mode(&mut self) -> bool {
        self.launch_selection
            .as_ref()
            .map(|selection| selection.dev_mode)
            .unwrap_or(false)
    }

    pub fn npr_gpu_meshes(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| stats.snapshot().world_3d_npr_gpu_realtime_meshes as rhai::INT)
            .unwrap_or(0)
    }

    pub fn npr_cpu_meshes(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| stats.snapshot().world_3d_npr_cpu_reference_meshes as rhai::INT)
            .unwrap_or(0)
    }

    pub fn npr_gpu_edges(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| stats.snapshot().world_3d_npr_gpu_realtime_enqueued_edges as rhai::INT)
            .unwrap_or(0)
    }

    pub fn npr_gpu_triangles(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| {
                stats
                    .snapshot()
                    .world_3d_npr_gpu_realtime_enqueued_triangles as rhai::INT
            })
            .unwrap_or(0)
    }

    pub fn npr_paths(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| stats.snapshot().world_3d_npr_paths as rhai::INT)
            .unwrap_or(0)
    }

    pub fn npr_path_states_capacity(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| {
                stats
                    .snapshot()
                    .world_3d_npr_gpu_realtime_path_states_capacity as rhai::INT
            })
            .unwrap_or(0)
    }

    pub fn npr_path_segments_capacity(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| {
                stats
                    .snapshot()
                    .world_3d_npr_gpu_realtime_path_segments_capacity as rhai::INT
            })
            .unwrap_or(0)
    }

    pub fn npr_stroke_segments_capacity(&mut self) -> rhai::INT {
        self.render_stats
            .as_ref()
            .map(|stats| {
                stats
                    .snapshot()
                    .world_3d_npr_gpu_realtime_stroke_segments_capacity as rhai::INT
            })
            .unwrap_or(0)
    }

    pub fn npr_debug_mode(&mut self) -> String {
        self.render_stats
            .as_ref()
            .map(|stats| stats.snapshot().world_3d_npr_gpu_realtime_debug_mode)
            .unwrap_or_default()
    }
}

pub fn runtime_window_backend(diagnostics: Option<&Arc<RuntimeDiagnostics>>) -> String {
    diagnostics
        .map(|diagnostics| diagnostics.window_backend.clone())
        .unwrap_or_default()
}

pub fn runtime_input_backend(diagnostics: Option<&Arc<RuntimeDiagnostics>>) -> String {
    diagnostics
        .map(|diagnostics| diagnostics.input_backend.clone())
        .unwrap_or_default()
}

pub fn runtime_render_backend(diagnostics: Option<&Arc<RuntimeDiagnostics>>) -> String {
    diagnostics
        .map(|diagnostics| diagnostics.render_backend.clone())
        .unwrap_or_default()
}

pub fn runtime_script_backend(diagnostics: Option<&Arc<RuntimeDiagnostics>>) -> String {
    diagnostics
        .map(|diagnostics| diagnostics.script_backend.clone())
        .unwrap_or_default()
}

pub fn runtime_capabilities(diagnostics: Option<&Arc<RuntimeDiagnostics>>) -> Vec<String> {
    diagnostics
        .map(|diagnostics| diagnostics.capabilities.clone())
        .unwrap_or_default()
}

pub fn runtime_plugins(diagnostics: Option<&Arc<RuntimeDiagnostics>>) -> Vec<String> {
    diagnostics
        .map(|diagnostics| diagnostics.plugin_names.clone())
        .unwrap_or_default()
}

pub fn runtime_services(diagnostics: Option<&Arc<RuntimeDiagnostics>>) -> Vec<String> {
    diagnostics
        .map(|diagnostics| diagnostics.service_names.clone())
        .unwrap_or_default()
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<RuntimeApi>("WorldRuntime")
        .register_fn("window_backend", RuntimeApi::window_backend)
        .register_fn("input_backend", RuntimeApi::input_backend)
        .register_fn("render_backend", RuntimeApi::render_backend)
        .register_fn("script_backend", RuntimeApi::script_backend)
        .register_fn("capabilities", RuntimeApi::capabilities)
        .register_fn("plugins", RuntimeApi::plugins)
        .register_fn("services", RuntimeApi::services)
        .register_fn("dev_mode", RuntimeApi::dev_mode)
        .register_fn("npr_gpu_meshes", RuntimeApi::npr_gpu_meshes)
        .register_fn("npr_cpu_meshes", RuntimeApi::npr_cpu_meshes)
        .register_fn("npr_gpu_edges", RuntimeApi::npr_gpu_edges)
        .register_fn("npr_gpu_triangles", RuntimeApi::npr_gpu_triangles)
        .register_fn("npr_paths", RuntimeApi::npr_paths)
        .register_fn(
            "npr_path_states_capacity",
            RuntimeApi::npr_path_states_capacity,
        )
        .register_fn(
            "npr_path_segments_capacity",
            RuntimeApi::npr_path_segments_capacity,
        )
        .register_fn(
            "npr_stroke_segments_capacity",
            RuntimeApi::npr_stroke_segments_capacity,
        )
        .register_fn("npr_debug_mode", RuntimeApi::npr_debug_mode);
}
