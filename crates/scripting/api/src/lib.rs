use std::sync::{Arc, Mutex};

use amigo_core::AmigoResult;

#[derive(Debug, Clone)]
pub struct ScriptRuntimeInfo {
    pub backend_name: &'static str,
    pub file_extension: &'static str,
}

pub trait ScriptRuntime: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;
    fn validate(&self, source: &str) -> AmigoResult<()>;
    fn execute(&self, source_name: &str, source: &str) -> AmigoResult<()>;
    fn unload(&self, source_name: &str) -> AmigoResult<()>;
    fn call_update(&self, source_name: &str, delta_seconds: f32) -> AmigoResult<()>;
    fn call_on_enter(&self, source_name: &str) -> AmigoResult<()>;
    fn call_on_exit(&self, source_name: &str) -> AmigoResult<()>;
    fn call_on_event(&self, source_name: &str, topic: &str, payload: &[String]) -> AmigoResult<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptCommand {
    pub namespace: String,
    pub name: String,
    pub arguments: Vec<String>,
}

impl ScriptCommand {
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<Vec<String>>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }

    pub fn ui_set_text(path: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new("ui", "set-text", vec![path.into(), value.into()])
    }

    pub fn ui_set_value(path: impl Into<String>, value: f32) -> Self {
        Self::new("ui", "set-value", vec![path.into(), value.to_string()])
    }

    pub fn ui_set_color(path: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new("ui", "set-color", vec![path.into(), value.into()])
    }

    pub fn ui_show(path: impl Into<String>) -> Self {
        Self::new("ui", "show", vec![path.into()])
    }

    pub fn ui_hide(path: impl Into<String>) -> Self {
        Self::new("ui", "hide", vec![path.into()])
    }

    pub fn ui_enable(path: impl Into<String>) -> Self {
        Self::new("ui", "enable", vec![path.into()])
    }

    pub fn ui_disable(path: impl Into<String>) -> Self {
        Self::new("ui", "disable", vec![path.into()])
    }

    pub fn audio_play(clip_name: impl Into<String>) -> Self {
        Self::new("audio", "play", vec![clip_name.into()])
    }

    pub fn audio_play_asset(asset_key: impl Into<String>) -> Self {
        Self::new("audio", "play-asset", vec![asset_key.into()])
    }

    pub fn audio_cue(cue_name: impl Into<String>) -> Self {
        Self::new("audio", "cue", vec![cue_name.into()])
    }

    pub fn scene_activate_set(set_id: impl Into<String>) -> Self {
        Self::new("scene", "activate-set", vec![set_id.into()])
    }

    pub fn audio_preload(clip_name: impl Into<String>) -> Self {
        Self::new("audio", "preload", vec![clip_name.into()])
    }

    pub fn audio_start_realtime(source: impl Into<String>) -> Self {
        Self::new("audio", "start-realtime", vec![source.into()])
    }

    pub fn audio_stop(source: impl Into<String>) -> Self {
        Self::new("audio", "stop", vec![source.into()])
    }

    pub fn audio_set_param(
        source: impl Into<String>,
        param: impl Into<String>,
        value: f32,
    ) -> Self {
        Self::new(
            "audio",
            "set-param",
            vec![source.into(), param.into(), value.to_string()],
        )
    }

    pub fn audio_set_volume(bus: impl Into<String>, value: f32) -> Self {
        Self::new("audio", "set-volume", vec![bus.into(), value.to_string()])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptEvent {
    pub topic: String,
    pub payload: Vec<String>,
}

impl ScriptEvent {
    pub fn new(topic: impl Into<String>, payload: impl Into<Vec<String>>) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevConsoleCommand {
    pub line: String,
}

impl DevConsoleCommand {
    pub fn new(line: impl Into<String>) -> Self {
        Self { line: line.into() }
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::{
        DevConsoleCommand, DevConsoleQueue, DevConsoleState, ScriptCommand, ScriptCommandQueue,
        ScriptEvent, ScriptEventQueue,
    };

    #[test]
    fn queues_script_commands_and_events() {
        let commands = ScriptCommandQueue::default();
        let events = ScriptEventQueue::default();

        commands.submit(ScriptCommand::new(
            "scene",
            "select",
            vec!["dev-shell".to_owned()],
        ));
        events.publish(ScriptEvent::new(
            "scene.selected",
            vec!["dev-shell".to_owned()],
        ));

        assert_eq!(commands.pending().len(), 1);
        assert_eq!(events.pending().len(), 1);
        assert_eq!(commands.drain().len(), 1);
        assert_eq!(events.drain().len(), 1);
    }

    #[test]
    fn queues_dev_console_commands() {
        let queue = DevConsoleQueue::default();

        queue.submit(DevConsoleCommand::new("help"));

        assert_eq!(queue.pending().len(), 1);
        assert_eq!(queue.drain().len(), 1);
    }

    #[test]
    fn stores_dev_console_history_and_output() {
        let state = DevConsoleState::default();

        state.record_command("help");
        state.write_line("available placeholder commands: help");

        assert_eq!(state.command_history(), vec!["help".to_owned()]);
        assert_eq!(
            state.output_lines(),
            vec!["available placeholder commands: help".to_owned()]
        );
    }

    #[test]
    fn builds_ui_script_commands() {
        assert_eq!(
            ScriptCommand::ui_set_text("playground-2d-ui-preview.subtitle", "Updated from Rhai"),
            ScriptCommand::new(
                "ui",
                "set-text",
                vec![
                    "playground-2d-ui-preview.subtitle".to_owned(),
                    "Updated from Rhai".to_owned(),
                ],
            )
        );
        assert_eq!(
            ScriptCommand::ui_set_value("playground-2d-ui-preview.hp-bar", 0.5),
            ScriptCommand::new(
                "ui",
                "set-value",
                vec![
                    "playground-2d-ui-preview.hp-bar".to_owned(),
                    "0.5".to_owned(),
                ],
            )
        );
        assert_eq!(
            ScriptCommand::ui_show("playground-2d-ui-preview.root"),
            ScriptCommand::new(
                "ui",
                "show",
                vec!["playground-2d-ui-preview.root".to_owned()],
            )
        );
        assert_eq!(
            ScriptCommand::ui_hide("playground-2d-ui-preview.root"),
            ScriptCommand::new(
                "ui",
                "hide",
                vec!["playground-2d-ui-preview.root".to_owned()],
            )
        );
        assert_eq!(
            ScriptCommand::ui_enable(
                "playground-2d-ui-preview.root.control-card.button-row.repair-button"
            ),
            ScriptCommand::new(
                "ui",
                "enable",
                vec![
                    "playground-2d-ui-preview.root.control-card.button-row.repair-button"
                        .to_owned()
                ],
            )
        );
        assert_eq!(
            ScriptCommand::ui_disable(
                "playground-2d-ui-preview.root.control-card.button-row.repair-button"
            ),
            ScriptCommand::new(
                "ui",
                "disable",
                vec![
                    "playground-2d-ui-preview.root.control-card.button-row.repair-button"
                        .to_owned()
                ],
            )
        );
        assert_eq!(
            ScriptCommand::audio_play("jump"),
            ScriptCommand::new("audio", "play", vec!["jump".to_owned()])
        );
        assert_eq!(
            ScriptCommand::audio_start_realtime("proximity-beep"),
            ScriptCommand::new("audio", "start-realtime", vec!["proximity-beep".to_owned()])
        );
    }
}
