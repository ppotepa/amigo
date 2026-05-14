use std::sync::Arc;

use amigo_core::{LaunchSelection, RuntimeDiagnostics};

use crate::bindings::common::string_array;

#[derive(Clone)]
pub struct RuntimeApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) diagnostics: Option<Arc<RuntimeDiagnostics>>,
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
