use amigo_overlay_api::{build_ui_layout_tree, UiOverlayDocument, UiOverlayNodeKind};
use amigo_panel_api::*;
use amigo_runtime::{Runtime, RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use amigo_runtime_control::RuntimeControlService;
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};

pub use amigo_scene::{
    ScenePanelHostDocument as PanelHost, ScenePanelReferenceDocument as PanelReference,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelConnectionSnapshot {
    pub generation: u64,
    pub process_id: Option<u32>,
    pub ready: bool,
    pub failure: Option<String>,
}

/// An interaction submitted by a transport-independent panel host.
///
/// The external egui process currently serializes equivalent messages over
/// stdio. Keeping this contract local to the panel service lets an embedded
/// host use the same validation and authored script-event routing without
/// pretending to be a separate process.
#[derive(Debug, Clone)]
pub enum PanelInteraction {
    Edit {
        panel_id: String,
        generation: u64,
        revision: u64,
        control: String,
        value: amigo_runtime_control::ControlValue,
    },
    Reset {
        panel_id: String,
        generation: u64,
        revision: u64,
        control: String,
    },
    Click {
        panel_id: String,
        generation: u64,
        revision: u64,
        control: String,
    },
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
    host: PanelHost,
    connection: Option<Connection>,
}
struct State {
    reported_error: Option<String>,
    executable: Option<PathBuf>,
    scene: Option<String>,
    generation: u64,
    panels: BTreeMap<String, Panel>,
    embedded_tabs: BTreeMap<String, String>,
    embedded_collapsed: BTreeMap<String, bool>,
    embedded_hovered: BTreeMap<String, String>,
    embedded_dropdowns: BTreeMap<String, bool>,
    embedded_dropdown_scrolls: BTreeMap<String, f32>,
    embedded_scroll_offsets: BTreeMap<String, f32>,
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
            embedded_tabs: BTreeMap::new(),
            embedded_collapsed: BTreeMap::new(),
            embedded_hovered: BTreeMap::new(),
            embedded_dropdowns: BTreeMap::new(),
            embedded_dropdown_scrolls: BTreeMap::new(),
            embedded_scroll_offsets: BTreeMap::new(),
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
    /// Returns the declared document and current bound values without choosing
    /// a windowing or rendering backend. Callers render this snapshot and send
    /// changes back through [`Self::apply_interaction`].
    pub fn snapshots(
        &self,
        controls: &RuntimeControlService,
        presets: &crate::PresetService,
    ) -> Result<Vec<PanelSnapshot>, String> {
        let state = self.state.lock().unwrap();
        state
            .panels
            .values()
            .filter(|panel| panel.host.includes_embedded())
            .map(|panel| {
                snapshot_panel(
                    state.generation,
                    panel.revision,
                    &panel.document,
                    controls,
                    presets,
                )
            })
            .collect()
    }
    pub fn embedded_overlays(
        &self,
        controls: &RuntimeControlService,
        presets: &crate::PresetService,
    ) -> Result<Vec<UiOverlayDocument>, String> {
        let state = self.state.lock().unwrap();
        let tabs = state.embedded_tabs.clone();
        let collapsed = state.embedded_collapsed.clone();
        let hovered = state.embedded_hovered.clone();
        let dropdowns = state.embedded_dropdowns.clone();
        let dropdown_scrolls = state.embedded_dropdown_scrolls.clone();
        let scroll_offsets = state.embedded_scroll_offsets.clone();
        drop(state);
        self.snapshots(controls, presets).map(|snapshots| {
            snapshots
                .iter()
                .map(|snapshot| {
                    crate::overlay(
                        snapshot,
                        &tabs,
                        &collapsed,
                        &hovered,
                        &dropdowns,
                        &dropdown_scrolls,
                        &scroll_offsets,
                    )
                })
                .collect()
        })
    }
    pub fn set_embedded_tab(&self, panel_id: &str, control_id: &str, tab_id: String) {
        self.state
            .lock()
            .unwrap()
            .embedded_tabs
            .insert(format!("{panel_id}:{control_id}"), tab_id);
    }
    pub fn set_embedded_collapsed(&self, panel_id: &str, control_id: &str, value: bool) {
        self.state
            .lock()
            .unwrap()
            .embedded_collapsed
            .insert(format!("{panel_id}:{control_id}"), value);
    }
    pub fn set_embedded_hovered(&self, panel_id: &str, value: Option<String>) {
        let mut state = self.state.lock().unwrap();
        match value {
            Some(value) => {
                state.embedded_hovered.insert(panel_id.to_owned(), value);
            }
            None => {
                state.embedded_hovered.remove(panel_id);
            }
        }
    }
    pub fn set_embedded_dropdown(&self, panel_id: &str, control_id: &str, value: bool) {
        self.state
            .lock()
            .unwrap()
            .embedded_dropdowns
            .insert(format!("{panel_id}:{control_id}"), value);
    }
    pub fn close_embedded_dropdowns(&self) {
        self.state.lock().unwrap().embedded_dropdowns.clear();
    }
    pub fn set_embedded_dropdown_scroll(
        &self,
        panel_id: &str,
        control_id: &str,
        value: f32,
        option_count: usize,
    ) {
        let max = option_count.saturating_sub(10) as f32;
        self.state
            .lock()
            .unwrap()
            .embedded_dropdown_scrolls
            .insert(format!("{panel_id}:{control_id}"), value.clamp(0.0, max));
    }
    pub fn set_embedded_scroll_offset(&self, panel_id: &str, value: f32, max_offset: f32) {
        self.state
            .lock()
            .unwrap()
            .embedded_scroll_offsets
            .insert(panel_id.to_owned(), value.clamp(0.0, max_offset.max(0.0)));
    }
    /// Applies an interaction against the exact scene/layout revision supplied
    /// by a host. This is deliberately shared by native and external hosts.
    pub fn apply_interaction(
        &self,
        interaction: PanelInteraction,
        controls: &RuntimeControlService,
        events: &ScriptEventQueue,
    ) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        let generation = state.generation;
        let (panel_id, control) = match &interaction {
            PanelInteraction::Edit {
                panel_id,
                revision: _,
                control,
                ..
            }
            | PanelInteraction::Reset {
                panel_id,
                revision: _,
                control,
                ..
            }
            | PanelInteraction::Click {
                panel_id,
                revision: _,
                control,
                ..
            } => (panel_id.clone(), control.clone()),
        };
        let panel = state
            .panels
            .get_mut(&panel_id)
            .ok_or_else(|| format!("unknown panel {panel_id}"))?;
        match interaction {
            PanelInteraction::Edit {
                generation: actual_generation,
                revision: actual_revision,
                value,
                ..
            } => apply_edit(
                &panel.document,
                generation,
                panel.revision,
                actual_generation,
                actual_revision,
                &control,
                value,
                controls,
                events,
            ),
            PanelInteraction::Reset {
                generation: actual_generation,
                revision: actual_revision,
                ..
            } => apply_reset(
                &panel.document,
                generation,
                panel.revision,
                actual_generation,
                actual_revision,
                &control,
                controls,
                events,
            ),
            PanelInteraction::Click {
                generation: actual_generation,
                revision: actual_revision,
                ..
            } => apply_click(
                &panel.document,
                generation,
                panel.revision,
                actual_generation,
                actual_revision,
                &control,
                controls,
                events,
            ),
        }
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
        state.embedded_tabs.clear();
        state.embedded_collapsed.clear();
        state.embedded_hovered.clear();
        state.embedded_dropdowns.clear();
        state.embedded_dropdown_scrolls.clear();
        state.embedded_scroll_offsets.clear();
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
                host: reference.host,
                connection: None,
            };
            if reference.auto_open && reference.host.includes_external() {
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
                            let result = apply_reset(
                                &panel.document,
                                generation,
                                panel.revision,
                                g,
                                revision,
                                &control,
                                controls,
                                events,
                            );
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
                            let result = apply_click(
                                &panel.document,
                                generation,
                                panel.revision,
                                g,
                                revision,
                                &control,
                                controls,
                                events,
                            );
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
                    match snapshot_panel(
                        generation,
                        panel.revision,
                        &panel.document,
                        controls,
                        presets,
                    ) {
                        Ok(snapshot) => {
                            *connection.latest.lock().unwrap() = Some(ServerMessage::Snapshot {
                                generation: snapshot.generation,
                                revision: snapshot.revision,
                                acknowledged: connection.last_request,
                                preset_names: snapshot.preset_names,
                                values: snapshot.values,
                            });
                        }
                        Err(e) => error = Some(e),
                    }
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
fn apply_reset(
    doc: &PanelDocument,
    generation: u64,
    revision: u64,
    actual_generation: u64,
    actual_revision: u64,
    id: &str,
    controls: &RuntimeControlService,
    events: &ScriptEventQueue,
) -> Result<(), String> {
    validate_epoch(generation, revision, actual_generation, actual_revision)?;
    doc.validate_bindings(&controls.registry_snapshot())?;
    validate_enabled(doc, id, controls)?;
    let node = doc
        .nodes()
        .into_iter()
        .find(|node| node.id.as_deref() == Some(id))
        .ok_or("unknown control")?;
    let path = node
        .value_bind
        .as_ref()
        .ok_or("control has no value binding")?;
    controls.reset(path).map_err(|error| error.to_string())?;
    if let Some(binding) = &node.on_change {
        let mut payload = binding.payload.clone();
        payload.push(path.clone());
        events.publish(ScriptEvent::new(binding.event.clone(), payload));
    }
    Ok(())
}
fn apply_click(
    doc: &PanelDocument,
    generation: u64,
    revision: u64,
    actual_generation: u64,
    actual_revision: u64,
    id: &str,
    controls: &RuntimeControlService,
    events: &ScriptEventQueue,
) -> Result<(), String> {
    validate_epoch(generation, revision, actual_generation, actual_revision)?;
    validate_enabled(doc, id, controls)?;
    let node = doc
        .nodes()
        .into_iter()
        .find(|node| node.id.as_deref() == Some(id))
        .ok_or("unknown control")?;
    let binding = node
        .on_click
        .as_ref()
        .ok_or("control has no click action")?;
    events.publish(ScriptEvent::new(
        binding.event.clone(),
        binding.payload.clone(),
    ));
    Ok(())
}
fn snapshot_panel(
    generation: u64,
    revision: u64,
    document: &PanelDocument,
    controls: &RuntimeControlService,
    presets: &crate::PresetService,
) -> Result<PanelSnapshot, String> {
    let registry = controls.registry_snapshot();
    document.validate_bindings(&registry)?;
    let paths = document.binding_paths().into_iter().collect::<Vec<_>>();
    let batch = controls
        .get_many(&paths)
        .map_err(|error| error.to_string())?;
    let mut values = BTreeMap::new();
    for path in paths {
        let property = registry
            .property(&path)
            .ok_or_else(|| format!("unknown binding {path}"))?;
        let value = batch
            .get(&path)
            .cloned()
            .ok_or_else(|| format!("missing value {path}"))?;
        values.insert(
            path.clone(),
            PropertySnapshot {
                path,
                value,
                value_type: property.value_type,
                writable: property.writable,
                range: property.range.clone(),
                description: property.description.clone(),
            },
        );
    }
    let preset_names = document
        .preset_domain_bind
        .as_ref()
        .and_then(|path| batch.get(path))
        .and_then(|value| value.as_string())
        .map(|domain| presets.list_for(domain))
        .unwrap_or_else(|| presets.list());
    Ok(PanelSnapshot {
        generation,
        revision,
        document: document.clone(),
        preset_names,
        values,
    })
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
    std::thread::spawn(move || loop {
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
    });
    std::thread::spawn(move || loop {
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
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::PreUpdate,
            "panel_embedded_input",
            |runtime| tick_embedded_panel_input(runtime),
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
fn tick_embedded_panel_input(runtime: &Runtime) -> amigo_core::AmigoResult<()> {
    let Some(viewport) = runtime
        .resolve::<amigo_ui::UiInputViewportState>()
        .and_then(|state| state.get())
    else {
        return Ok(());
    };
    let Some(input) = runtime.resolve::<amigo_ui::UiInputService>() else {
        return Ok(());
    };
    let input = input.snapshot();
    let Some(mouse) = input.mouse_position else {
        return Ok(());
    };
    let panels = runtime.required::<PanelService>()?;
    let controls = runtime.required::<RuntimeControlService>()?;
    let events = runtime.required::<ScriptEventQueue>()?;
    let presets = runtime.required::<crate::PresetService>()?;
    let snapshots = panels
        .snapshots(controls.as_ref(), presets.as_ref())
        .map_err(amigo_core::AmigoError::Message)?;
    let state = panels.state.lock().unwrap();
    let tabs = state.embedded_tabs.clone();
    let collapsed = state.embedded_collapsed.clone();
    let dropdowns = state.embedded_dropdowns.clone();
    let dropdown_scrolls = state.embedded_dropdown_scrolls.clone();
    let scroll_offsets = state.embedded_scroll_offsets.clone();
    drop(state);
    for snapshot in &snapshots {
        panels.set_embedded_hovered(&snapshot.document.id, None);
    }
    if input.mouse_left_released {
        panels.close_embedded_dropdowns();
    }
    for snapshot in snapshots.iter().rev() {
        let overlay = crate::overlay(
            snapshot,
            &tabs,
            &collapsed,
            &Default::default(),
            &dropdowns,
            &dropdown_scrolls,
            &scroll_offsets,
        );
        let layout = build_ui_layout_tree(viewport, &overlay);
        let Some(path) = amigo_ui::hit_test_ui_layout(&layout, mouse.x, mouse.y) else {
            continue;
        };
        let Some(layout_node) = amigo_ui::find_ui_layout_node(&layout, &path) else {
            continue;
        };
        if input.mouse_wheel_y.abs() > f32::EPSILON
            && !expanded_dropdown_owns_wheel(&layout_node.node.kind)
        {
            if let Some(max_offset) = embedded_scroll_area_max_offset_at(&layout, mouse.x, mouse.y)
            {
                panels.set_embedded_scroll_offset(
                    &snapshot.document.id,
                    scroll_offsets
                        .get(&snapshot.document.id)
                        .copied()
                        .unwrap_or_default()
                        - input.mouse_wheel_y * 36.0,
                    max_offset,
                );
                break;
            }
        }
        let Some(id) = layout_node.node.id.as_deref() else {
            continue;
        };
        let hovered = crate::choice_value_for_id(snapshot, id)
            .is_some()
            .then(|| id.to_owned());
        panels.set_embedded_hovered(&snapshot.document.id, hovered);
        if input.mouse_left_released {
            if let Some((control, value)) = crate::choice_value_for_id(snapshot, id) {
                panels
                    .apply_interaction(
                        PanelInteraction::Edit {
                            panel_id: snapshot.document.id.clone(),
                            generation: snapshot.generation,
                            revision: snapshot.revision,
                            control,
                            value: amigo_runtime_control::ControlValue::String(value),
                        },
                        controls.as_ref(),
                        events.as_ref(),
                    )
                    .map_err(amigo_core::AmigoError::Message)?;
                break;
            }
        }
        let Some(node) = crate::node_by_id(snapshot, id) else {
            continue;
        };
        if node.kind == amigo_scene::SceneUiNodeTypeComponentDocument::GroupBox
            && input.mouse_left_released
        {
            let value = !crate::group_is_collapsed(snapshot, node, &collapsed);
            panels.set_embedded_collapsed(&snapshot.document.id, id, value);
            break;
        }
        if let UiOverlayNodeKind::Dropdown {
            options,
            expanded,
            scroll_offset,
            ..
        } = &layout_node.node.kind
        {
            if *expanded && input.mouse_wheel_y.abs() > f32::EPSILON {
                panels.set_embedded_dropdown_scroll(
                    &snapshot.document.id,
                    id,
                    *scroll_offset - input.mouse_wheel_y * 0.65,
                    options.len(),
                );
                break;
            }
            if input.mouse_left_released {
                if !expanded {
                    panels.set_embedded_dropdown(&snapshot.document.id, id, true);
                } else if let Some(value) =
                    dropdown_value_from_mouse(layout_node.rect, options, *scroll_offset, mouse.y)
                {
                    panels
                        .apply_interaction(
                            PanelInteraction::Edit {
                                panel_id: snapshot.document.id.clone(),
                                generation: snapshot.generation,
                                revision: snapshot.revision,
                                control: id.to_owned(),
                                value: amigo_runtime_control::ControlValue::String(value),
                            },
                            controls.as_ref(),
                            events.as_ref(),
                        )
                        .map_err(amigo_core::AmigoError::Message)?;
                }
            }
            break;
        }
        let interaction = match &layout_node.node.kind {
            UiOverlayNodeKind::TabView { tabs, .. } if input.mouse_left_released => {
                amigo_overlay_api::tab_view_tab_from_mouse(
                    layout_node.rect,
                    &layout_node.node,
                    tabs,
                    mouse.x,
                    mouse.y,
                )
                .map(|tab| {
                    panels.set_embedded_tab(&snapshot.document.id, id, tab);
                    return None;
                })
                .flatten()
            }
            UiOverlayNodeKind::Button { .. } if input.mouse_left_released => {
                Some(PanelInteraction::Click {
                    panel_id: snapshot.document.id.clone(),
                    generation: snapshot.generation,
                    revision: snapshot.revision,
                    control: id.into(),
                })
            }
            UiOverlayNodeKind::Toggle { checked, .. } if input.mouse_left_released => {
                crate::editable_value(snapshot, node).map(|_| PanelInteraction::Edit {
                    panel_id: snapshot.document.id.clone(),
                    generation: snapshot.generation,
                    revision: snapshot.revision,
                    control: id.into(),
                    value: amigo_runtime_control::ControlValue::Bool(!checked),
                })
            }
            UiOverlayNodeKind::OptionSet {
                selected, options, ..
            } if input.mouse_left_released => {
                crate::editable_value(snapshot, node).and_then(|_| {
                    next_option(options, selected).map(|value| PanelInteraction::Edit {
                        panel_id: snapshot.document.id.clone(),
                        generation: snapshot.generation,
                        revision: snapshot.revision,
                        control: id.into(),
                        value: amigo_runtime_control::ControlValue::String(value),
                    })
                })
            }
            UiOverlayNodeKind::Slider { min, max, step, .. } if input.mouse_left_down => {
                crate::editable_value(snapshot, node).and_then(|property| {
                    let width = layout_node.rect.width;
                    (width > f32::EPSILON).then(|| {
                        let t = ((mouse.x - layout_node.rect.x) / width).clamp(0.0, 1.0);
                        let raw = min + (max - min) * t;
                        let value = (raw / step.max(f32::EPSILON)).round() * step.max(f32::EPSILON);
                        let value = match property.value {
                            amigo_runtime_control::ControlValue::I64(_) => {
                                amigo_runtime_control::ControlValue::I64(value.round() as i64)
                            }
                            amigo_runtime_control::ControlValue::U64(_) => {
                                amigo_runtime_control::ControlValue::U64(
                                    value.max(0.0).round() as u64
                                )
                            }
                            _ => amigo_runtime_control::ControlValue::F64(value as f64),
                        };
                        PanelInteraction::Edit {
                            panel_id: snapshot.document.id.clone(),
                            generation: snapshot.generation,
                            revision: snapshot.revision,
                            control: id.into(),
                            value,
                        }
                    })
                })
            }
            _ => None,
        };
        if let Some(interaction) = interaction {
            panels
                .apply_interaction(interaction, controls.as_ref(), events.as_ref())
                .map_err(amigo_core::AmigoError::Message)?;
        }
        break;
    }
    Ok(())
}
fn next_option(options: &[String], selected: &str) -> Option<String> {
    (!options.is_empty()).then(|| {
        let current = options
            .iter()
            .position(|option| option == selected)
            .unwrap_or(0);
        options[(current + 1) % options.len()].clone()
    })
}

fn expanded_dropdown_owns_wheel(kind: &UiOverlayNodeKind) -> bool {
    matches!(kind, UiOverlayNodeKind::Dropdown { expanded: true, .. })
}

fn embedded_scroll_area_max_offset_at(
    layout: &amigo_overlay_api::UiLayoutNode,
    mouse_x: f32,
    mouse_y: f32,
) -> Option<f32> {
    let contains = mouse_x >= layout.rect.x
        && mouse_x <= layout.rect.x + layout.rect.width
        && mouse_y >= layout.rect.y
        && mouse_y <= layout.rect.y + layout.rect.height;
    if !contains {
        return None;
    }
    if let UiOverlayNodeKind::ScrollArea { .. } = layout.node.kind {
        let content_bottom = layout
            .children
            .iter()
            .map(|child| child.rect.y + child.rect.height)
            .fold(layout.rect.y + layout.rect.height, f32::max);
        return layout
            .node
            .id
            .as_deref()
            .map(|_| (content_bottom - (layout.rect.y + layout.rect.height)).max(0.0));
    }
    layout
        .children
        .iter()
        .find_map(|child| embedded_scroll_area_max_offset_at(child, mouse_x, mouse_y))
}

fn dropdown_value_from_mouse(
    rect: amigo_overlay_api::UiRect,
    options: &[String],
    scroll_offset: f32,
    mouse_y: f32,
) -> Option<String> {
    let row_height = 38.0_f32.min(rect.height.max(0.0));
    if row_height <= f32::EPSILON {
        return None;
    }
    let row = ((mouse_y - rect.y) / row_height).floor();
    let index = (scroll_offset + row - 1.0).floor();
    (row >= 1.0 && index.is_finite() && index >= 0.0)
        .then(|| options.get(index as usize).cloned())
        .flatten()
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
