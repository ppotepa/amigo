use std::sync::{Arc, Mutex};

use amigo_core::AmigoResult;

use crate::runtime::{ScriptRuntime, ScriptSourceContext};
use crate::types::{
    DevConsoleCommand, ScriptCommand, ScriptComponentDefinition, ScriptEvent, ScriptParams,
};

#[derive(Debug, Default)]
pub struct ScriptCommandQueue {
    commands: Mutex<Vec<ScriptCommand>>,
}

impl ScriptCommandQueue {
    pub fn submit(&self, command: ScriptCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("script command queue mutex should not be poisoned");
        commands.push(command);
    }

    pub fn pending(&self) -> Vec<ScriptCommand> {
        let commands = self
            .commands
            .lock()
            .expect("script command queue mutex should not be poisoned");
        commands.clone()
    }

    pub fn drain(&self) -> Vec<ScriptCommand> {
        let mut commands = self
            .commands
            .lock()
            .expect("script command queue mutex should not be poisoned");
        commands.drain(..).collect()
    }
}

#[derive(Debug, Default)]
pub struct ScriptEventQueue {
    events: Mutex<Vec<ScriptEvent>>,
}

impl ScriptEventQueue {
    pub fn publish(&self, event: ScriptEvent) {
        let mut events = self
            .events
            .lock()
            .expect("script event queue mutex should not be poisoned");
        events.push(event);
    }

    pub fn pending(&self) -> Vec<ScriptEvent> {
        let events = self
            .events
            .lock()
            .expect("script event queue mutex should not be poisoned");
        events.clone()
    }

    pub fn drain(&self) -> Vec<ScriptEvent> {
        let mut events = self
            .events
            .lock()
            .expect("script event queue mutex should not be poisoned");
        events.drain(..).collect()
    }
}

#[derive(Debug, Default)]
pub struct DevConsoleQueue {
    commands: Mutex<Vec<DevConsoleCommand>>,
}

impl DevConsoleQueue {
    pub fn submit(&self, command: DevConsoleCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("dev console queue mutex should not be poisoned");
        commands.push(command);
    }

    pub fn pending(&self) -> Vec<DevConsoleCommand> {
        let commands = self
            .commands
            .lock()
            .expect("dev console queue mutex should not be poisoned");
        commands.clone()
    }

    pub fn drain(&self) -> Vec<DevConsoleCommand> {
        let mut commands = self
            .commands
            .lock()
            .expect("dev console queue mutex should not be poisoned");
        commands.drain(..).collect()
    }
}

#[derive(Debug, Default)]
struct DevConsoleStateInner {
    command_history: Vec<String>,
    output_lines: Vec<String>,
}

#[derive(Debug, Default)]
pub struct DevConsoleState {
    inner: Mutex<DevConsoleStateInner>,
}

impl DevConsoleState {
    pub fn record_command(&self, line: impl Into<String>) {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        inner.command_history.push(line.into());
    }

    pub fn write_line(&self, line: impl Into<String>) {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        inner.output_lines.push(line.into());
    }

    pub fn command_history(&self) -> Vec<String> {
        let inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        inner.command_history.clone()
    }

    pub fn output_lines(&self) -> Vec<String> {
        let inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        inner.output_lines.clone()
    }
}

#[derive(Debug, Default)]
pub struct ScriptLifecycleState {
    active_scene: Mutex<Option<String>>,
}

impl ScriptLifecycleState {
    pub fn active_scene(&self) -> Option<String> {
        self.active_scene
            .lock()
            .expect("script lifecycle mutex should not be poisoned")
            .clone()
    }

    pub fn set_active_scene(&self, scene_id: Option<String>) {
        *self
            .active_scene
            .lock()
            .expect("script lifecycle mutex should not be poisoned") = scene_id;
    }
}

#[derive(Clone)]
pub struct ScriptRuntimeService {
    runtime: Arc<dyn ScriptRuntime>,
}

impl ScriptRuntimeService {
    pub fn new<T>(runtime: T) -> Self
    where
        T: ScriptRuntime + 'static,
    {
        Self {
            runtime: Arc::new(runtime),
        }
    }

    pub fn backend_name(&self) -> &'static str {
        self.runtime.backend_name()
    }

    pub fn file_extension(&self) -> &'static str {
        self.runtime.file_extension()
    }

    pub fn supports_extension(&self, extension: &str) -> bool {
        extension.eq_ignore_ascii_case(self.file_extension())
    }

    pub fn validate_source(&self, source: &str) -> AmigoResult<()> {
        self.runtime.validate(source)
    }

    pub fn set_source_context(&self, context: ScriptSourceContext) -> AmigoResult<()> {
        self.runtime.set_source_context(context)
    }

    pub fn execute_source(&self, source_name: &str, source: &str) -> AmigoResult<()> {
        self.runtime.execute(source_name, source)
    }

    pub fn unload_source(&self, source_name: &str) -> AmigoResult<()> {
        self.runtime.unload(source_name)
    }

    pub fn call_update(&self, source_name: &str, delta_seconds: f32) -> AmigoResult<()> {
        self.runtime.call_update(source_name, delta_seconds)
    }

    pub fn call_on_enter(&self, source_name: &str) -> AmigoResult<()> {
        self.runtime.call_on_enter(source_name)
    }

    pub fn call_on_exit(&self, source_name: &str) -> AmigoResult<()> {
        self.runtime.call_on_exit(source_name)
    }

    pub fn call_on_event(
        &self,
        source_name: &str,
        topic: &str,
        payload: &[String],
    ) -> AmigoResult<()> {
        self.runtime.call_on_event(source_name, topic, payload)
    }

    pub fn call_event_function(
        &self,
        source_name: &str,
        function_name: &str,
        topic: &str,
        payload: &[String],
    ) -> AmigoResult<()> {
        self.runtime
            .call_event_function(source_name, function_name, topic, payload)
    }

    pub fn call_component_on_attach(
        &self,
        source_name: &str,
        entity_name: &str,
        params: &ScriptParams,
    ) -> AmigoResult<()> {
        self.runtime
            .call_component_on_attach(source_name, entity_name, params)
    }

    pub fn call_component_update(
        &self,
        source_name: &str,
        entity_name: &str,
        params: &ScriptParams,
        delta_seconds: f32,
    ) -> AmigoResult<()> {
        self.runtime
            .call_component_update(source_name, entity_name, params, delta_seconds)
    }

    pub fn call_component_on_detach(
        &self,
        source_name: &str,
        entity_name: &str,
        params: &ScriptParams,
    ) -> AmigoResult<()> {
        self.runtime
            .call_component_on_detach(source_name, entity_name, params)
    }
}

#[derive(Debug, Default)]
pub struct ScriptComponentService {
    components: Mutex<Vec<ScriptComponentDefinition>>,
}

impl ScriptComponentService {
    pub fn queue(&self, component: ScriptComponentDefinition) {
        let mut components = self
            .components
            .lock()
            .expect("script component service mutex should not be poisoned");
        components.retain(|existing| existing.source_name != component.source_name);
        components.push(component);
    }

    pub fn components(&self) -> Vec<ScriptComponentDefinition> {
        self.components
            .lock()
            .expect("script component service mutex should not be poisoned")
            .clone()
    }

    pub fn clear(&self) {
        self.components
            .lock()
            .expect("script component service mutex should not be poisoned")
            .clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptTraceEntry {
    pub label: String,
    pub values: Vec<(String, String)>,
}

#[derive(Debug, Default)]
pub struct ScriptTraceService {
    entries: Mutex<Vec<ScriptTraceEntry>>,
    stack: Mutex<Vec<ScriptTraceEntry>>,
}

impl ScriptTraceService {
    pub fn begin(&self, label: impl Into<String>) {
        self.stack
            .lock()
            .expect("script trace stack mutex should not be poisoned")
            .push(ScriptTraceEntry {
                label: label.into(),
                values: Vec::new(),
            });
    }

    pub fn value(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut stack = self
            .stack
            .lock()
            .expect("script trace stack mutex should not be poisoned");
        if let Some(entry) = stack.last_mut() {
            entry.values.push((key.into(), value.into()));
        }
    }

    pub fn end(&self) -> bool {
        let Some(entry) = self
            .stack
            .lock()
            .expect("script trace stack mutex should not be poisoned")
            .pop()
        else {
            return false;
        };
        self.entries
            .lock()
            .expect("script trace entries mutex should not be poisoned")
            .push(entry);
        true
    }

    pub fn entries(&self) -> Vec<ScriptTraceEntry> {
        self.entries
            .lock()
            .expect("script trace entries mutex should not be poisoned")
            .clone()
    }

    pub fn clear(&self) {
        self.entries
            .lock()
            .expect("script trace entries mutex should not be poisoned")
            .clear();
        self.stack
            .lock()
            .expect("script trace stack mutex should not be poisoned")
            .clear();
    }
}
