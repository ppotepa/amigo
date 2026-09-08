use amigo_panel_api::*;
use amigo_runtime::{Runtime, RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use amigo_runtime_control::RuntimeControlService;
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

pub use amigo_scene::ScenePanelReferenceDocument as PanelReference;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelConnectionSnapshot {
    pub generation: u64,
    pub process_id: Option<u32>,
    pub ready: bool,
    pub failure: Option<String>,
}
#[derive(Default, Deserialize)]
struct ScenePanels {
    #[serde(default)]
    panels: Vec<PanelReference>,
}

struct Connection {
    started: Instant,
    transport_error: Arc<Mutex<Option<String>>>,
    ready: bool,
    last_request: u64,
    child: Child,
    outgoing: mpsc::SyncSender<ServerMessage>,
    incoming: mpsc::Receiver<ClientMessage>,
    latest: Arc<Mutex<Option<ServerMessage>>>,
}
impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
struct Panel {
    failure: Option<String>,
    path: PathBuf,
    source: String,
    document: PanelDocument,
    revision: u64,
    error: Option<String>,
    connection: Option<Connection>,
}
struct State {
    reported_error: Option<String>,
    executable: Option<PathBuf>,
    scene: Option<String>,
    generation: u64,
    panels: BTreeMap<String, Panel>,
    last_poll: Instant,
    last_snapshot: Instant,
    error: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            reported_error: None,
            executable: None,
            scene: None,
            generation: 0,
            panels: BTreeMap::new(),
            last_poll: Instant::now(),
            last_snapshot: Instant::now(),
            error: None,
        }
    }
}

#[derive(Default)]
pub struct PanelService {
    state: Mutex<State>,
    watches: amigo_hot_reload::HotReloadService,
}
impl PanelService {
    pub fn connection_snapshot(&self, id: &str) -> Option<PanelConnectionSnapshot> {
        let state = self.state.lock().unwrap();
        let panel = state.panels.get(id)?;
        Some(PanelConnectionSnapshot {
            generation: state.generation,
            process_id: panel.connection.as_ref().map(|c| c.child.id()),
            ready: panel.connection.as_ref().is_some_and(|c| c.ready),
            failure: panel.failure.clone(),
        })
    }
    pub fn enable_host(&self, executable: PathBuf) {
        let mut state = self.state.lock().unwrap();
        state.executable = Some(executable);
        state.scene = None;
    }
    pub fn last_error(&self) -> Option<String> {
        self.state.lock().unwrap().error.clone()
    }
    fn report_diagnostics(&self, runtime: &Runtime) {
        let error = {
            let mut state = self.state.lock().unwrap();
            if state.reported_error == state.error {
                return;
            }
            state.reported_error = state.error.clone();
            state.error.clone()
        };
        if let Some(error) = error {
            let message = format!("[panels.host] {error}");
            eprintln!("{message}");
            if let Some(log) = runtime.resolve::<amigo_scripting_api::RunLogService>() {
                log.write_runtime(&message);
            }
            if let Some(console) = runtime.resolve::<amigo_scripting_api::DevConsoleState>() {
                console.write_line_with_level(
                    message,
                    amigo_scripting_api::DevConsoleOutputLevel::Error,
                );
            }
        }
    }
    pub fn open(&self, id: &str) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        let exe = state
            .executable
            .clone()
            .ok_or("panels require an interactive host")?;
        let generation = state.generation;
        let panel = state
            .panels
            .get_mut(id)
            .ok_or_else(|| format!("unknown panel {id}"))?;
        if panel.connection.is_none() {
            panel.failure = None;
            match connect(&exe, generation, panel) {
                Ok(connection) => panel.connection = Some(connection),
                Err(error) => {
                    panel.failure = Some(error.clone());
                    state.error = Some(error.clone());
                    return Err(error);
                }
            }
        }
        Ok(())
    }
    pub fn close(&self, id: &str) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state
            .panels
            .get_mut(id)
            .ok_or_else(|| format!("unknown panel {id}"))?
            .connection = None;
        Ok(())
    }
    pub fn load_scene(
        &self,
        key: Option<String>,
        root: &Path,
        scene_path: &Path,
    ) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if state.scene == key {
            return Ok(());
        }
        state.panels.clear();
        self.watches.sync_assets(vec![]);
        state.scene = None;
        state.generation += 1;
        if key.is_none() {
            return Ok(());
        }
        let source = std::fs::read_to_string(scene_path).map_err(|e| e.to_string())?;
        let declared: ScenePanels = serde_yaml::from_str(&source).map_err(|e| e.to_string())?;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let mut pending = BTreeMap::new();
        let mut auto_open = Vec::new();
        for reference in declared.panels {
            if pending.contains_key(&reference.id) {
                return Err(format!("duplicate panel {}", reference.id));
            }
            let path = scene_path
                .parent()
                .unwrap_or(root.as_path())
                .join(&reference.layout)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.starts_with(&root) {
                return Err("panel layout escapes mod root".into());
            }
            let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let document: PanelDocument =
                serde_yaml::from_str(&source).map_err(|e| format!("{}: {e}", path.display()))?;
            document.validate()?;
            if document.id != reference.id {
                return Err("panel reference and document ids differ".into());
            }
            let panel = Panel {
                failure: None,
                path,
                source,
                document,
                revision: 1,
                error: None,
                connection: None,
            };
            if reference.auto_open {
                auto_open.push(reference.id.clone());
            }
            pending.insert(reference.id, panel);
        }
        if let Some(exe) = &state.executable {
            for id in auto_open {
                let panel = pending.get_mut(&id).unwrap();
                match connect(exe, state.generation, panel) {
                    Ok(connection) => panel.connection = Some(connection),
                    Err(error) => panel.failure = Some(error),
                }
            }
        }
        self.watches.sync_assets(
            pending
                .iter()
                .map(|(id, p)| amigo_hot_reload::AssetWatch {
                    asset_key: format!("panel:{id}"),
                    path: p.path.clone(),
                })
                .collect(),
        );
        state.panels = pending;
        state.scene = key;
        state.error = None;
        Ok(())
    }
    pub fn tick(
        &self,
        controls: &RuntimeControlService,
        events: &ScriptEventQueue,
        presets: &crate::PresetService,
    ) {
        let mut state = self.state.lock().unwrap();
        let generation = state.generation;
        let poll = state.last_poll.elapsed() >= Duration::from_millis(250);
        if poll {
            state.last_poll = Instant::now();
        }
        let changed = if poll {
            self.watches.poll_changes()
        } else {
            vec![]
        };
        let publish = state.last_snapshot.elapsed() >= Duration::from_secs_f64(1.0 / 30.0);
        if publish {
            state.last_snapshot = Instant::now();
        }
        let mut error = presets.take_error();
        for panel in state.panels.values_mut() {
            if changed.iter().any(|change| change.watch.path == panel.path) {
                match std::fs::read_to_string(&panel.path) {
                    Ok(source) if source != panel.source => {
                        let parsed = serde_yaml::from_str::<PanelDocument>(&source)
                            .map_err(|e| e.to_string())
                            .and_then(|doc| {
                                doc.validate_bindings(&controls.registry_snapshot())?;
                                if doc.id != panel.document.id {
                                    return Err("panel id changed".into());
                                }
                                Ok(doc)
                            });
                        match parsed {
                            Ok(doc) => {
                                panel.error = None;
                                panel.document = doc;
                                panel.source = source;
                                panel.revision += 1;
                                if let Some(c) = &panel.connection {
                                    let _ =
                                        c.outgoing.try_send(document_message(generation, panel));
                                }
                            }
                            Err(e) => {
                                panel.error = Some(format!("{}: {e}", panel.path.display()));
                            }
                        }
                    }
                    Err(e) => panel.error = Some(format!("{}: {e}", panel.path.display())),
                    Ok(_) => {
                        if panel.error.take().is_some() {
                            if let Some(c) = &panel.connection {
                                let _ = c
                                    .outgoing
                                    .try_send(ServerMessage::Diagnostic(String::new()));
                            }
                        }
                    }
                }
            }
            if panel.error.is_some() {
                error = panel.error.clone();
            }
            let mut closed = false;
            if let Some(connection) = &mut panel.connection {
                while let Ok(message) = connection.incoming.try_recv() {
                    let (request, result) = match message {
                        ClientMessage::Hello { version } => {
                            if version != PROTOCOL_VERSION {
                                panel.failure = Some(format!(
                                    "unsupported panel protocol {version}; expected {PROTOCOL_VERSION}. Rebuild the host executable."
                                ));
                                closed = true;
                                break;
                            }
                            connection.ready = true;
                            continue;
                        }
                        ClientMessage::Close => {
                            closed = true;
                            break;
                        }
                        ClientMessage::Edit {
                            request,
                            generation: g,
                            revision,
                            control,
                            value,
                        } => {
                            if !connection.ready {
                                panel.failure =
                                    Some("panel sent an edit before its handshake".into());
                                closed = true;
                                break;
                            }
                            if request <= connection.last_request {
                                closed = true;
                                break;
                            }
                            connection.last_request = request;
                            (
                                request,
                                apply_edit(
                                    &panel.document,
                                    generation,
                                    panel.revision,
                                    g,
                                    revision,
                                    &control,
                                    value,
                                    controls,
                                    events,
                                ),
                            )
                        }
                        ClientMessage::Reset {
                            request,
                            generation: g,
                            revision,
                            control,
                        } => {
                            if !connection.ready || request <= connection.last_request {
                                panel.failure = Some("invalid reset request/handshake".into());
                                closed = true;
                                break;
                            }
                            connection.last_request = request;
                            let result = validate_epoch(generation, panel.revision, g, revision)
                                .and_then(|_| {
                                    panel
                                        .document
                                        .validate_bindings(&controls.registry_snapshot())?;
                                    validate_enabled(&panel.document, &control, controls)?;
                                    let node = panel
                                        .document
                                        .nodes()
                                        .into_iter()
                                        .find(|n| n.id.as_deref() == Some(&control))
                                        .ok_or("unknown control")?;
                                    let path = node
                                        .value_bind
                                        .as_ref()
                                        .ok_or("control has no value binding")?;
                                    controls.reset(path).map_err(|e| e.to_string())?;
                                    if let Some(binding) = &node.on_change {
                                        let mut payload = binding.payload.clone();
                                        payload.push(path.clone());
                                        events.publish(ScriptEvent::new(
                                            binding.event.clone(),
                                            payload,
                                        ));
                                    }
                                    Ok(())
                                });
                            (request, result)
                        }
                        ClientMessage::Click {
                            request,
                            generation: g,
                            revision,
                            control,
                        } => {
                            if !connection.ready {
                                panel.failure =
                                    Some("panel sent an action before its handshake".into());
                                closed = true;
                                break;
                            }
                            if request <= connection.last_request {
                                closed = true;
                                break;
                            }
                            connection.last_request = request;
                            let result = validate_epoch(generation, panel.revision, g, revision)
                                .and_then(|_| {
                                    let node = panel
                                        .document
                                        .nodes()
                                        .into_iter()
                                        .find(|n| n.id.as_deref() == Some(&control))
                                        .ok_or("unknown control")?;
                                    validate_enabled(&panel.document, &control, controls)?;
                                    let binding = node
                                        .on_click
                                        .as_ref()
                                        .ok_or("control has no click action")?;
                                    events.publish(ScriptEvent::new(
                                        binding.event.clone(),
                                        binding.payload.clone(),
                                    ));
                                    Ok(())
                                });
                            (request, result)
                        }
                    };
                    if connection
                        .outgoing
                        .try_send(ServerMessage::Result {
                            request,
                            error: result.err(),
                        })
                        .is_err()
                    {
                        closed = true;
                        break;
                    }
                }
                if !closed {
                    match connection.child.try_wait() {
                        Ok(Some(status)) => {
                            closed = true;
                            if !connection.ready || !status.success() {
                                panel.failure = Some(format!(
                                    "panel process exited {status} (handshake: {}). Rebuild the host and reopen the panel.",
                                    connection.ready
                                ));
                            }
                        }
                        Err(e) => {
                            closed = true;
                            panel.failure = Some(format!("cannot inspect panel process: {e}"));
                        }
                        Ok(None) => {
                            if !connection.ready
                                && connection.started.elapsed() >= Duration::from_secs(5)
                            {
                                closed = true;
                                panel.failure=Some("panel handshake timed out after 5 seconds. The host must handle --runtime-panel-client; rebuild it and reopen the panel.".into());
                            }
                            if let Some(e) = connection.transport_error.lock().unwrap().take() {
                                closed = true;
                                panel.failure =
                                    Some(format!("panel transport failed: {e}. Reopen the panel."));
                            }
                        }
                    }
                }
                if publish && connection.ready && !closed {
                    let registry = controls.registry_snapshot();
                    if let Err(e) = panel.document.validate_bindings(&registry) {
                        error = Some(e);
                    }
                    let mut values = BTreeMap::new();
                    let paths = panel
                        .document
                        .binding_paths()
                        .into_iter()
                        .collect::<Vec<_>>();
                    let batch = controls.get_many(&paths);
                    if let Err(e) = &batch {
                        error = Some(e.to_string());
                    }
                    for path in &paths {
                        if let Some(property) = registry.property(path) {
                            match batch
                                .as_ref()
                                .ok()
                                .and_then(|values| values.get(path))
                                .cloned()
                                .ok_or_else(|| format!("missing value {path}"))
                            {
                                Ok(value) => {
                                    values.insert(
                                        path.clone(),
                                        PropertySnapshot {
                                            path: path.clone(),
                                            value,
                                            value_type: property.value_type,
                                            writable: property.writable,
                                            range: property.range.clone(),
                                            description: property.description.clone(),
                                        },
                                    );
                                }
                                Err(e) => error = Some(e.to_string()),
                            }
                        } else {
                            error = Some(format!("unknown binding {path}"));
                        }
                    }
                    *connection.latest.lock().unwrap() = Some(ServerMessage::Snapshot {
                        generation,
                        revision: panel.revision,
                        acknowledged: connection.last_request,
                        preset_names: panel
                            .document
                            .preset_domain_bind
                            .as_ref()
                            .and_then(|path| batch.as_ref().ok()?.get(path)?.as_string())
                            .map(|domain| presets.list_for(domain))
                            .unwrap_or_else(|| presets.list()),
                        values,
                    });
                }
                if let Some(e) = &error {
                    let _ = connection
                        .outgoing
                        .try_send(ServerMessage::Diagnostic(e.clone()));
                }
            }
            if closed {
                panel.connection = None;
            }
            if let Some(failure) = &panel.failure {
                error = Some(format!("panel '{}': {failure}", panel.document.id));
            }
        }
        state.error = error;
    }
}
fn validate_epoch(g: u64, r: u64, actual_g: u64, actual_r: u64) -> Result<(), String> {
    if g != actual_g || r != actual_r {
        Err("stale scene or layout".into())
    } else {
        Ok(())
    }
}
fn apply_edit(
    doc: &PanelDocument,
    g: u64,
    r: u64,
    actual_g: u64,
    actual_r: u64,
    id: &str,
    value: amigo_runtime_control::ControlValue,
    controls: &RuntimeControlService,
    events: &ScriptEventQueue,
) -> Result<(), String> {
    validate_epoch(g, r, actual_g, actual_r)?;
    doc.validate_bindings(&controls.registry_snapshot())?;
    validate_enabled(doc, id, controls)?;
    let node = doc
        .nodes()
        .into_iter()
        .find(|n| n.id.as_deref() == Some(id))
        .ok_or("unknown control")?;
    let path = node
        .value_bind
        .as_ref()
        .ok_or("control has no value binding")?;
    if let Some(number) = value.as_f64() {
        if !number.is_finite()
            || node.min.is_some_and(|min| number < f64::from(min))
            || node.max.is_some_and(|max| number > f64::from(max))
        {
            return Err("value outside control range".into());
        }
    }
    if !node.options.is_empty()
        && !value
            .as_string()
            .is_some_and(|v| node.options.iter().any(|o| o == v))
    {
        return Err("unknown option".into());
    }
    controls.set(path, value).map_err(|e| e.to_string())?;
    if let Some(binding) = &node.on_change {
        let mut payload = binding.payload.clone();
        payload.push(path.clone());
        events.publish(ScriptEvent::new(binding.event.clone(), payload));
    }
    Ok(())
}
fn validate_enabled(
    doc: &PanelDocument,
    id: &str,
    controls: &RuntimeControlService,
) -> Result<(), String> {
    fn visit(
        node: &amigo_scene::SceneUiNodeComponentDocument,
        id: &str,
        controls: &RuntimeControlService,
        parents_enabled: bool,
    ) -> Result<Option<bool>, String> {
        let mut enabled = parents_enabled;
        for path in [&node.visible_bind, &node.enabled_bind]
            .into_iter()
            .flatten()
        {
            enabled &= controls
                .get(path)
                .map_err(|e| e.to_string())?
                .as_bool()
                .ok_or("enable/visibility binding must be boolean")?;
        }
        if node.id.as_deref() == Some(id) {
            return Ok(Some(enabled));
        }
        for child in &node.children {
            if let Some(found) = visit(child, id, controls, enabled)? {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }
    match visit(&doc.root, id, controls, true)? {
        Some(true) => Ok(()),
        Some(false) => Err("control is disabled or hidden".into()),
        None => Err("unknown control".into()),
    }
}
fn document_message(generation: u64, panel: &Panel) -> ServerMessage {
    ServerMessage::Document {
        version: PROTOCOL_VERSION,
        generation,
        revision: panel.revision,
        document: panel.document.clone(),
    }
}
fn connect(exe: &Path, generation: u64, panel: &Panel) -> Result<Connection, String> {
    let mut command = Command::new(exe);
    command
        .arg("--runtime-panel-client")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    spawn_connection(&mut command, generation, panel).map_err(|e| {
        format!(
            "could not start {} --runtime-panel-client: {e}",
            exe.display()
        )
    })
}

fn spawn_connection(
    command: &mut Command,
    generation: u64,
    panel: &Panel,
) -> Result<Connection, String> {
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let mut input = child.stdin.take().unwrap();
    let mut output = child.stdout.take().unwrap();
    let (outgoing, rx) = mpsc::sync_channel(64);
    let (tx, incoming) = mpsc::sync_channel(64);
    let latest: Arc<Mutex<Option<ServerMessage>>> = Arc::default();
    let latest_writer = latest.clone();
    let transport_error: Arc<Mutex<Option<String>>> = Arc::default();
    let writer_error = transport_error.clone();
    let reader_error = transport_error.clone();
    std::thread::spawn(move || {
        loop {
            match rx.recv_timeout(Duration::from_millis(10)) {
                Ok(message) => {
                    if let Err(e) = write_message(&mut input, &message) {
                        *writer_error.lock().unwrap() = Some(e.to_string());
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if let Some(message) = latest_writer.lock().unwrap().take() {
                if let Err(e) = write_message(&mut input, &message) {
                    *writer_error.lock().unwrap() = Some(e.to_string());
                    break;
                }
            }
        }
    });
    std::thread::spawn(move || {
        loop {
            match read_message(&mut output) {
                Ok(message) => {
                    let close = matches!(message, ClientMessage::Close);
                    if tx.send(message).is_err() || close {
                        break;
                    }
                }
                Err(e) => {
                    *reader_error.lock().unwrap() = Some(e.to_string());
                    break;
                }
            }
        }
    });
    let connection = Connection {
        started: Instant::now(),
        transport_error,
        ready: false,
        last_request: 0,
        child,
        outgoing,
        incoming,
        latest,
    };
    connection
        .outgoing
        .send(document_message(generation, panel))
        .map_err(|e| e.to_string())?;
    Ok(connection)
}

pub struct PanelsPlugin;
impl RuntimePlugin for PanelsPlugin {
    fn name(&self) -> &'static str {
        "amigo-panels"
    }
    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(PanelService::default())?;
        registry.register(crate::PresetService::default())?;
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::PreUpdate,
            "panel_commands",
            |runtime| tick_panels(runtime),
        );
        Ok(())
    }
}
fn tick_panels(runtime: &Runtime) -> amigo_core::AmigoResult<()> {
    let panels = runtime.required::<PanelService>()?;
    if let (Some(session), Some(mods)) = (
        runtime.resolve::<amigo_session::SceneSessionService>(),
        runtime.resolve::<amigo_modding::ModCatalog>(),
    ) {
        let snapshot = session.snapshot();
        if let Some(doc) = snapshot.loaded_scene_document() {
            if let Some(source) = mods.mod_by_id(&doc.source_mod) {
                let key = format!(
                    "{}:{}:{}",
                    doc.source_mod,
                    doc.scene_id,
                    snapshot.lifecycle_summary().clear_count
                );
                if let Err(e) = panels.load_scene(
                    Some(key),
                    &source.root_path,
                    &source.root_path.join(&doc.relative_path),
                ) {
                    panels.state.lock().unwrap().error = Some(e);
                    panels.report_diagnostics(runtime);
                    return Ok(());
                }
                let project = source
                    .root_path
                    .parent()
                    .and_then(|p| p.parent())
                    .unwrap_or(Path::new("."));
                runtime.required::<crate::PresetService>()?.set_directory(
                    project
                        .join(".amigo/presets")
                        .join(&doc.source_mod)
                        .join(&doc.scene_id),
                );
            }
        } else {
            let _ = panels.load_scene(None, Path::new("."), Path::new("."));
        }
    }
    panels.tick(
        runtime.required::<RuntimeControlService>()?.as_ref(),
        runtime.required::<ScriptEventQueue>()?.as_ref(),
        runtime.required::<crate::PresetService>()?.as_ref(),
    );
    panels.report_diagnostics(runtime);
    Ok(())
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
