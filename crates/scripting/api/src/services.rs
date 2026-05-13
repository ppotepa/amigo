use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use amigo_core::AmigoResult;

use crate::{DevConsoleInputBuffer, DevConsoleInputSnapshot};
use crate::runtime::{
    DevConsoleEvalResult, DevConsoleScriptContext, ScriptRuntime, ScriptSourceContext,
};
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DevConsoleOutputLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevConsoleOutputLine {
    pub text: String,
    pub level: DevConsoleOutputLevel,
}

#[derive(Debug, Default)]
struct DevConsoleStateInner {
    open: bool,
    input: DevConsoleInputBuffer,
    input_clipboard: String,
    command_history: Vec<String>,
    history_cursor: Option<usize>,
    output_scroll_offset: usize,
    output_entries: Vec<DevConsoleOutputLine>,
}

#[derive(Debug, Default)]
pub struct DevConsoleState {
    inner: Mutex<DevConsoleStateInner>,
    run_log: Mutex<Option<Arc<RunLogService>>>,
}

impl DevConsoleState {
    pub fn attach_run_log(&self, run_log: Arc<RunLogService>) {
        *self
            .run_log
            .lock()
            .expect("dev console run log mutex should not be poisoned") = Some(run_log);
    }

    pub fn write_console_log(&self, line: impl AsRef<str>) {
        if let Some(run_log) = self
            .run_log
            .lock()
            .expect("dev console run log mutex should not be poisoned")
            .clone()
        {
            run_log.write_console(line.as_ref());
        }
    }

    pub fn is_open(&self) -> bool {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .open
    }

    pub fn set_open(&self, open: bool) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .open = open;
    }

    pub fn toggle_open(&self) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        inner.open = !inner.open;
        inner.open
    }

    pub fn input(&self) -> String {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .text()
            .to_owned()
    }

    pub fn input_snapshot(&self) -> DevConsoleInputSnapshot {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .snapshot()
    }

    pub fn input_cursor(&self) -> usize {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .cursor()
    }

    pub fn set_input(&self, value: impl Into<String>) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .set_text(value);
    }

    pub fn set_input_with_cursor(&self, value: impl Into<String>, cursor: usize) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .set_text_with_cursor(value, cursor);
    }

    pub fn push_input_text(&self, text: &str) {
        self.insert_input_text(text);
    }

    pub fn insert_input_text(&self, text: &str) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .insert_text(text);
    }

    pub fn backspace_input(&self) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .backspace();
    }

    pub fn delete_input(&self) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .delete();
    }

    pub fn move_input_left(&self, select: bool, word: bool) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .move_left(select, word);
    }

    pub fn move_input_right(&self, select: bool, word: bool) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .move_right(select, word);
    }

    pub fn move_input_home(&self, select: bool) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .move_home(select);
    }

    pub fn move_input_end(&self, select: bool) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .move_end(select);
    }

    pub fn select_all_input(&self) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .select_all();
    }

    pub fn copy_input_selection(&self) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");

        let Some(value) = inner.input.selected_text() else {
            return false;
        };

        inner.input_clipboard = value;
        true
    }

    pub fn cut_input_selection(&self) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");

        let Some(value) = inner.input.cut_selection() else {
            return false;
        };

        inner.input_clipboard = value;
        true
    }

    pub fn paste_input_clipboard(&self) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");

        if inner.input_clipboard.is_empty() {
            return false;
        }

        let value = inner.input_clipboard.clone();
        inner.input.insert_text(&value);
        true
    }

    pub fn clear_input(&self) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .input
            .clear();
    }

    pub fn clear_output(&self) {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        inner.output_entries.clear();
        inner.output_scroll_offset = 0;
    }

    pub fn output_tail(&self, max_lines: usize) -> Vec<String> {
        self.output_window(max_lines)
            .into_iter()
            .map(|entry| entry.text)
            .collect()
    }

    pub fn output_entries(&self) -> Vec<DevConsoleOutputLine> {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .output_entries
            .clone()
    }

    pub fn output_window(&self, max_lines: usize) -> Vec<DevConsoleOutputLine> {
        let inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        output_window_for(&inner.output_entries, max_lines, inner.output_scroll_offset)
    }

    pub fn scroll_output(&self, delta_rows: isize) {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        let max_offset = inner.output_entries.len().saturating_sub(1);
        if delta_rows > 0 {
            inner.output_scroll_offset = inner
                .output_scroll_offset
                .saturating_add(delta_rows as usize)
                .min(max_offset);
        } else {
            inner.output_scroll_offset = inner
                .output_scroll_offset
                .saturating_sub(delta_rows.unsigned_abs());
        }
    }

    pub fn reset_output_scroll(&self) {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .output_scroll_offset = 0;
    }

    pub fn output_scroll_offset(&self) -> usize {
        self.inner
            .lock()
            .expect("dev console state mutex should not be poisoned")
            .output_scroll_offset
    }

    pub fn history_previous(&self) -> Option<String> {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        if inner.command_history.is_empty() {
            return None;
        }
        let next = inner
            .history_cursor
            .map(|cursor| cursor.saturating_sub(1))
            .unwrap_or_else(|| inner.command_history.len().saturating_sub(1));
        inner.history_cursor = Some(next);
        inner.command_history.get(next).cloned()
    }

    pub fn history_next(&self) -> Option<String> {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        let Some(cursor) = inner.history_cursor else {
            return None;
        };
        let next = cursor + 1;
        if next >= inner.command_history.len() {
            inner.history_cursor = None;
            return Some(String::new());
        }
        inner.history_cursor = Some(next);
        inner.command_history.get(next).cloned()
    }

    pub fn record_command(&self, line: impl Into<String>) {
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        let line = line.into();
        if !line.trim().is_empty() {
            inner.command_history.push(line.clone());
            push_output_entries(
                &mut inner,
                format!("> {line}"),
                DevConsoleOutputLevel::Command,
            );
            drop(inner);
            self.write_console_log(format!("input raw={line:?}"));
            let mut inner = self
                .inner
                .lock()
                .expect("dev console state mutex should not be poisoned");
            inner.history_cursor = None;
            return;
        }
        inner.history_cursor = None;
    }

    pub fn write_line(&self, line: impl Into<String>) {
        self.write_line_with_level(line, DevConsoleOutputLevel::Info);
    }

    pub fn write_line_with_level(&self, line: impl Into<String>, level: DevConsoleOutputLevel) {
        let line = line.into();
        let mut inner = self
            .inner
            .lock()
            .expect("dev console state mutex should not be poisoned");
        push_output_entries(&mut inner, line.clone(), level);
        drop(inner);
        self.write_console_log(format!("output level={level:?} text={line:?}"));
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
        inner
            .output_entries
            .iter()
            .map(|entry| entry.text.clone())
            .collect()
    }
}

#[derive(Debug)]
pub struct RunLogService {
    run_id: String,
    runtime_log_path: PathBuf,
    console_log_path: PathBuf,
    runtime_log: Mutex<File>,
    console_log: Mutex<File>,
}

impl RunLogService {
    pub fn new(log_directory: impl AsRef<Path>) -> AmigoResult<Self> {
        let run_id = new_run_log_id();
        Self::new_with_run_id(log_directory, run_id)
    }

    pub fn new_with_run_id(log_directory: impl AsRef<Path>, run_id: impl Into<String>) -> AmigoResult<Self> {
        let run_id = run_id.into();
        let log_directory = log_directory.as_ref();
        std::fs::create_dir_all(log_directory).map_err(|error| {
            amigo_core::AmigoError::Message(format!(
                "failed to create run log directory `{}`: {error}",
                log_directory.display()
            ))
        })?;

        let runtime_log_path = log_directory.join(format!("{run_id}.runtime.log"));
        let console_log_path = log_directory.join(format!("{run_id}.console.log"));
        let runtime_log = open_run_log_file(&runtime_log_path)?;
        let console_log = open_run_log_file(&console_log_path)?;

        let service = Self {
            run_id,
            runtime_log_path,
            console_log_path,
            runtime_log: Mutex::new(runtime_log),
            console_log: Mutex::new(console_log),
        };
        service.write_runtime("runtime log opened");
        service.write_console("console log opened");
        Ok(service)
    }

    pub fn default_for_process() -> AmigoResult<Self> {
        Self::new(PathBuf::from("target").join("amigo-runs"))
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    pub fn runtime_log_path(&self) -> &Path {
        &self.runtime_log_path
    }

    pub fn console_log_path(&self) -> &Path {
        &self.console_log_path
    }

    pub fn write_runtime(&self, line: impl AsRef<str>) {
        write_run_log_line(&self.runtime_log, line.as_ref());
    }

    pub fn write_console(&self, line: impl AsRef<str>) {
        write_run_log_line(&self.console_log, line.as_ref());
    }
}

fn open_run_log_file(path: &Path) -> AmigoResult<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| {
            amigo_core::AmigoError::Message(format!(
                "failed to open run log `{}`: {error}",
                path.display()
            ))
        })
}

fn write_run_log_line(log: &Mutex<File>, line: &str) {
    if let Ok(mut file) = log.lock() {
        let _ = writeln!(file, "{} {}", run_log_timestamp(), line);
    }
}

fn new_run_log_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("run-{millis}")
}

fn run_log_timestamp() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("t={millis}")
}

fn push_output_entries(
    inner: &mut DevConsoleStateInner,
    text: String,
    level: DevConsoleOutputLevel,
) {
    let mut pushed = false;
    for line in text.lines() {
        inner.output_entries.push(DevConsoleOutputLine {
            text: line.to_owned(),
            level,
        });
        pushed = true;
    }
    if !pushed {
        inner.output_entries.push(DevConsoleOutputLine {
            text: String::new(),
            level,
        });
    }
    if text.ends_with('\n') {
        inner.output_entries.push(DevConsoleOutputLine {
            text: String::new(),
            level,
        });
    }
    inner.output_scroll_offset = 0;
}

fn output_window_for(
    entries: &[DevConsoleOutputLine],
    max_lines: usize,
    scroll_offset: usize,
) -> Vec<DevConsoleOutputLine> {
    if max_lines == 0 || entries.is_empty() {
        return Vec::new();
    }
    let end = entries
        .len()
        .saturating_sub(scroll_offset.min(entries.len()));
    let start = end.saturating_sub(max_lines);
    entries[start..end].to_vec()
}

#[derive(Debug, Default)]
pub struct ScriptLifecycleState {
    active_scene: Mutex<Option<String>>,
    active_scripts: Mutex<Vec<crate::types::ActiveScriptRef>>,
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

    pub fn active_scripts(&self) -> Vec<crate::types::ActiveScriptRef> {
        self.active_scripts
            .lock()
            .expect("script lifecycle mutex should not be poisoned")
            .clone()
    }

    pub fn set_active_scripts(&self, scripts: Vec<crate::types::ActiveScriptRef>) {
        *self
            .active_scripts
            .lock()
            .expect("script lifecycle mutex should not be poisoned") = scripts;
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

    pub fn eval_console(
        &self,
        context: DevConsoleScriptContext,
        source: &str,
    ) -> AmigoResult<DevConsoleEvalResult> {
        self.runtime.eval_console(context, source)
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

