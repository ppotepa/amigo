//! Scene-owned companion processes and loopback transport.
use amigo_playground_api::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use tungstenite::{
    Message,
    handshake::server::{Request, Response},
};

const COMPANION_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaygroundClientBootstrap {
    pub native: Option<amigo_playground_native::NativeBootstrap>,
    pub endpoint: String,
    pub token: String,
    pub playground: PlaygroundId,
    pub client: String,
    pub label: String,
    pub version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    Refresh,
    FrameAck { ack: PlaygroundFrameAck },
    Action { action: PlaygroundActionEnvelope },
    Viewport { request: PlaygroundViewportRequest },
    Close,
}

struct Companion {
    #[cfg(windows)]
    native: Option<Arc<amigo_playground_native::NativeLink>>,
    child: Child,
    stop: Arc<AtomicBool>,
    incoming: mpsc::Receiver<ClientMessage>,
    outgoing: mpsc::SyncSender<PlaygroundEvent>,
    frames: Arc<PlaygroundFrameQueue>,
    viewport: Option<PlaygroundViewportRequest>,
    input_id: u64,
    worker: Option<thread::JoinHandle<()>>,
}

impl Drop for Companion {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Some(native) = &self.native { native.close(); }
        self.stop.store(true, Ordering::Release);
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Default)]
struct CompanionState {
    executable: Option<PathBuf>,
    scene: Option<String>,
    companions: BTreeMap<PlaygroundId, Companion>,
    error: Option<String>,
    owns_primary_window: bool,
}

#[derive(Default)]
pub struct PlaygroundCompanionService(Mutex<CompanionState>);

#[derive(Debug, Clone)]
pub struct PlaygroundConnectionSnapshot {
    pub id: String,
    pub process_id: u32,
    pub ready: bool,
}

pub(crate) struct PlaygroundsPlugin;
impl amigo_runtime::RuntimePlugin for PlaygroundsPlugin {
    fn name(&self) -> &'static str {
        "amigo-playgrounds"
    }
    fn register(
        &self,
        registry: &mut amigo_runtime::ServiceRegistry,
    ) -> amigo_core::AmigoResult<()> {
        registry.register(PlaygroundHostService::default())?;
        registry.register(PlaygroundCompanionService::default())?;
        registry
            .required::<amigo_runtime::SystemRegistry>()?
            .register_fn(
                amigo_runtime::SystemPhase::PostUpdate,
                "playground_companions",
                |runtime| {
                    let companions = runtime.required::<PlaygroundCompanionService>()?;
                    let host = runtime.required::<PlaygroundHostService>()?;
                    let result = (|| -> Result<(), String> {
                        let Some(session) = runtime.resolve::<amigo_session::SceneSessionService>()
                        else {
                            return Ok(());
                        };
                        let snapshot = session.snapshot();
                        if let Some(doc) = snapshot.loaded_scene_document() {
                            let key = format!(
                                "{}:{}:{}",
                                doc.source_mod,
                                doc.scene_id,
                                snapshot.lifecycle_summary().clear_count
                            );
                            {
                                let state = companions.0.lock().unwrap();
                                if state.executable.is_none() || state.scene.as_ref() == Some(&key)
                                {
                                    return Ok(());
                                }
                            }
                            let mods = runtime
                                .required::<amigo_modding::ModCatalog>()
                                .map_err(|e| e.to_string())?;
                            let source = mods
                                .mod_by_id(&doc.source_mod)
                                .ok_or("missing owning mod")?;
                            let scene_path = source.root_path.join(&doc.relative_path);
                            #[derive(Deserialize)]
                            struct References {
                                #[serde(default)]
                                playgrounds: Vec<amigo_scene::ScenePlaygroundReferenceDocument>,
                            }
                            let references: References = serde_yaml::from_slice(
                                &std::fs::read(&scene_path).map_err(|e| e.to_string())?,
                            )
                            .map_err(|e| e.to_string())?;
                            // Close the previous scene before opening domain state for the next one.
                            companions.load_scene(None, &[], host.clone())?;
                            for reference in &references.playgrounds {
                                host.open_scene(
                                    &PlaygroundId(reference.id.clone()),
                                    &source.root_path,
                                    &scene_path,
                                )?;
                            }
                            companions.load_scene(
                                Some(key),
                                &references.playgrounds,
                                host.clone(),
                            )?;
                        } else {
                            companions.load_scene(None, &[], host.clone())?;
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        companions.report_error(error);
                    }
                    companions.tick(
                        &host,
                        runtime
                            .resolve::<amigo_scripting_api::ScriptEventQueue>()
                            .as_deref(),
                    );
                    Ok(())
                },
            );
        Ok(())
    }
}

pub fn enable_playground_companions(
    runtime: &amigo_runtime::Runtime,
    executable: PathBuf,
) -> amigo_core::AmigoResult<()> {
    runtime
        .required::<PlaygroundCompanionService>()?
        .enable(executable);
    Ok(())
}

impl PlaygroundCompanionService {
    pub fn owns_primary_window(&self) -> bool {
        self.0.lock().unwrap().owns_primary_window
    }

    pub fn primary_window_closed(&self) -> bool {
        let state = self.0.lock().unwrap();
        state.owns_primary_window && state.companions.is_empty()
    }
    #[cfg(windows)]
    pub(crate) fn native(&self, id: &PlaygroundId) -> Option<Arc<amigo_playground_native::NativeLink>> {
        self.0.lock().unwrap().companions.get(id).and_then(|c| c.native.clone())
    }
    pub fn connections(&self) -> Vec<PlaygroundConnectionSnapshot> {
        self.0
            .lock()
            .unwrap()
            .companions
            .iter()
            .map(|(id, companion)| PlaygroundConnectionSnapshot {
                id: id.0.clone(),
                process_id: companion.child.id(),
                ready: companion.viewport.is_some(),
            })
            .collect()
    }

    pub fn close(&self, id: &str) -> bool {
        self.0
            .lock()
            .unwrap()
            .companions
            .remove(&PlaygroundId(id.into()))
            .is_some()
    }
    pub(crate) fn frame_queue(&self, id: &PlaygroundId) -> Option<Arc<PlaygroundFrameQueue>> {
        self.0
            .lock()
            .unwrap()
            .companions
            .get(id)
            .map(|c| c.frames.clone())
    }
    pub(crate) fn capture_submitted(&self, id: &PlaygroundId) {
        if let Some(request) = self
            .0
            .lock()
            .unwrap()
            .companions
            .get_mut(id)
            .and_then(|c| c.viewport.as_mut())
        {
            request.high_quality_capture = false;
        }
    }
    pub(crate) fn report_error(&self, error: String) {
        let mut state = self.0.lock().unwrap();
        if state.error.as_ref() != Some(&error) {
            eprintln!("playground: {error}");
            for companion in state.companions.values() {
                if companion
                    .outgoing
                    .try_send(PlaygroundEvent::Domain {
                        name: "viewport_failed".into(),
                        payload: serde_json::json!({"message":error}),
                    })
                    .is_err()
                {
                    companion.stop.store(true, Ordering::Release);
                }
            }
        }
        state.error = Some(error);
    }
    pub(crate) fn send_viewport_error(&self, id: &PlaygroundId, generation: u64, message: String) {
        eprintln!("playground: {message}");
        if let Some(companion) = self.0.lock().unwrap().companions.get(id) {
            if companion.outgoing.try_send(PlaygroundEvent::Domain {
                name: "viewport_failed".into(),
                payload: serde_json::json!({"generation":generation,"message":message}),
            }).is_err() {
                companion.stop.store(true, Ordering::Release);
            }
        }
    }
    pub(crate) fn send_viewport_diagnostics(&self, id: &PlaygroundId, stats: serde_json::Value) {
        if let Some(companion) = self.0.lock().unwrap().companions.get(id) {
            if companion
                .outgoing
                .try_send(PlaygroundEvent::Domain {
                    name: "viewport_diagnostics".into(),
                    payload: stats,
                })
                .is_err()
            {
                companion.stop.store(true, Ordering::Release);
            }
        }
    }
    pub fn enable(&self, executable: PathBuf) {
        self.0.lock().unwrap().executable = Some(executable);
    }
    pub fn error(&self) -> Option<String> {
        self.0.lock().unwrap().error.clone()
    }
    pub fn viewport(&self, id: &PlaygroundId) -> Option<PlaygroundViewportRequest> {
        self.0
            .lock()
            .unwrap()
            .companions
            .get(id)
            .and_then(|c| c.viewport)
    }
    pub(crate) fn input_id(&self, id: &PlaygroundId) -> u64 {
        self.0.lock().unwrap().companions.get(id).map_or(0, |c| c.input_id)
    }
    pub fn load_scene(
        &self,
        key: Option<String>,
        references: &[amigo_scene::ScenePlaygroundReferenceDocument],
        host: Arc<PlaygroundHostService>,
    ) -> Result<(), String> {
        let mut state = self.0.lock().unwrap();
        if state.scene == key {
            return Ok(());
        }
        state.companions.clear();
        state.scene = key;
        state.error = None;
        state.owns_primary_window = false;
        let Some(executable) = state.executable.clone() else {
            return Ok(());
        };
        state.owns_primary_window = references.iter().any(|reference| reference.auto_open);
        let mut pending = BTreeMap::new();
        for reference in references.iter().filter(|r| r.auto_open) {
            let id = PlaygroundId(reference.id.clone());
            if pending.contains_key(&id) {
                return Err(format!("duplicate playground {}", id.0));
            }
            let descriptor = host
                .descriptor(&id)
                .ok_or_else(|| format!("unregistered playground {}", id.0))?;
            pending.insert(id, spawn_companion(&executable, descriptor, host.clone())?);
        }
        state.companions = pending;
        Ok(())
    }
    pub fn tick(
        &self,
        host: &PlaygroundHostService,
        scripts: Option<&amigo_scripting_api::ScriptEventQueue>,
    ) {
        let mut state = self.0.lock().unwrap();
        let mut closed = vec![];
        let mut failure = None;
        for (id, companion) in &mut state.companions {
            if let Some(status) = companion.child.try_wait().ok().flatten() {
                if !status.success() {
                    failure = Some(format!("playground {} exited: {status}", id.0));
                }
                closed.push(id.clone());
                continue;
            }
            if companion.stop.load(Ordering::Acquire) {
                closed.push(id.clone());
                continue;
            }
            for message in companion.incoming.try_iter().take(64) {
                match message {
                    ClientMessage::Refresh => {
                        if let Ok(event) = host.refresh(id) { let _ = companion.outgoing.try_send(event); }
                    }
                    ClientMessage::Action { action } => {
                        let request_id = action.request_id;
                        let response = host.dispatch(id, action);
                        if matches!(&response, PlaygroundEvent::Accepted { .. }) { companion.input_id = request_id; }
                        if companion
                            .outgoing
                            .try_send(response)
                            .is_err()
                        {
                            failure = Some("playground control queue full".into());
                            closed.push(id.clone());
                            break;
                        }
                    }
                    ClientMessage::Viewport { request } => {
                        if request.effective_size([1280, 720]).is_ok() {
                            companion.frames.set_generation(request.generation);
                            companion.viewport = Some(request);
                        }
                    }
                    ClientMessage::FrameAck { ack } => { companion.frames.acknowledge(&ack); }
                    ClientMessage::Close => {
                        closed.push(id.clone());
                        break;
                    }
                }
            }
            match host.poll(id) {
                Ok(events) => {
                    for event in events {
                        publish_script_event(scripts, &event);
                        if companion.outgoing.try_send(event).is_err() {
                            closed.push(id.clone());
                            failure = Some("playground control queue full".into());
                            break;
                        }
                    }
                }
                Err(error) => {
                    failure = Some(error);
                    closed.push(id.clone());
                }
            }
        }
        for id in closed {
            state.companions.remove(&id);
        }
        for descriptor in host.descriptors() {
            if !state.companions.contains_key(&descriptor.id) {
                if let Ok(events) = host.poll(&descriptor.id) {
                    for event in events {
                        publish_script_event(scripts, &event);
                    }
                }
            }
        }
        if failure.is_some() {
            state.error = failure;
        }
    }
}

fn publish_script_event(
    queue: Option<&amigo_scripting_api::ScriptEventQueue>,
    event: &PlaygroundEvent,
) {
    if let (Some(queue), PlaygroundEvent::Domain { name, payload }) = (queue, event) {
        queue.publish(amigo_scripting_api::ScriptEvent::new(
            name.clone(),
            vec![payload.to_string()],
        ));
    }
}

fn spawn_companion(
    executable: &Path,
    descriptor: PlaygroundDescriptor,
    host: Arc<PlaygroundHostService>,
) -> Result<Companion, String> {
    let listener =
        TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut entropy = [0u8; 32];
    getrandom::fill(&mut entropy).map_err(|e| e.to_string())?;
    let token = entropy
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    #[cfg(windows)]
    let native_session = amigo_playground_native::native_server().ok();
    let bootstrap = PlaygroundClientBootstrap {
        #[cfg(windows)]
        native: native_session.as_ref().map(|(bootstrap, _)| bootstrap.clone()),
        #[cfg(not(windows))]
        native: None,
        endpoint: format!(
            "ws://{}/",
            listener.local_addr().map_err(|e| e.to_string())?
        ),
        token: token.clone(),
        playground: descriptor.id.clone(),
        client: descriptor.client,
        label: descriptor.label,
        version: PLAYGROUND_PROTOCOL_VERSION,
    };
    let mut command = Command::new(executable);
    command
        .arg("--runtime-playground-client")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    if let Some((_, native)) = &native_session { native.peer_pid.store(child.id(), Ordering::Release); }
    let encoded = serde_json::to_vec(&bootstrap).map_err(|e| e.to_string())?;
    let mut input = child.stdin.take().ok_or("missing child stdin")?;
    let sent = input
        .write_all(&(encoded.len() as u32).to_le_bytes())
        .and_then(|_| input.write_all(&encoded));
    drop(input);
    if let Err(error) = sent {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error.to_string());
    }
    let stop = Arc::new(AtomicBool::new(false));
    let frames = Arc::new(PlaygroundFrameQueue::default());
    let (outgoing, rx) = mpsc::sync_channel(64);
    let (tx, incoming) = mpsc::sync_channel(64);
    let worker_stop = stop.clone();
    let worker_frames = frames.clone();
    let worker = thread::spawn(move || {
        let started = std::time::Instant::now();
        let origin = if cfg!(windows) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        };
        let mut guard = PlaygroundHandshakeGuard::new(descriptor.id.clone(), token, origin.into());
        while !worker_stop.load(Ordering::Acquire)
            && started.elapsed() < COMPANION_HANDSHAKE_TIMEOUT
        {
            match listener.accept() {
                Ok((stream, peer)) if peer.ip().is_loopback() => {
                    match accept_client(stream, origin, &mut guard) {
                        Ok(mut socket) => {
                            let result = serve_client(
                                &mut socket,
                                &descriptor.id,
                                &host,
                                &rx,
                                &tx,
                                &worker_frames,
                                &worker_stop,
                            );
                            if let Err(error) = result {
                                eprintln!("playground transport: {error}");
                            }
                            break;
                        }
                        Err(error) => eprintln!("playground handshake: {error}"),
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5))
                }
                Err(_) => break,
                _ => {}
            }
        }
        worker_stop.store(true, Ordering::Release);
    });
    Ok(Companion {
        #[cfg(windows)]
        native: native_session.map(|(_, native)| native),
        child,
        stop,
        incoming,
        outgoing,
        frames,
        viewport: None,
        input_id: 0,
        worker: Some(worker),
    })
}

fn accept_client(
    stream: TcpStream,
    origin: &str,
    guard: &mut PlaygroundHandshakeGuard,
) -> Result<tungstenite::WebSocket<TcpStream>, String> {
    stream.set_nonblocking(false).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_millis(100)))
        .map_err(|e| e.to_string())?;
    let config = tungstenite::protocol::WebSocketConfig::default()
        .max_message_size(Some(256 * 1024))
        .max_frame_size(Some(256 * 1024));
    let mut socket = tungstenite::accept_hdr_with_config(
        stream,
        |request: &Request, response: Response| {
            if request
                .headers()
                .get("origin")
                .and_then(|v| v.to_str().ok())
                != Some(origin)
            {
                return Err(tungstenite::http::Response::builder()
                    .status(403)
                    .body(Some("origin rejected".into()))
                    .unwrap());
            }
            Ok(response)
        },
        Some(config),
    )
    .map_err(|e| e.to_string())?;
    let hello = socket.read().map_err(|e| e.to_string())?;
    let hello: PlaygroundHandshake =
        serde_json::from_str(hello.to_text().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    guard.accept(&hello, origin)?;
    socket
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(5)))
        .map_err(|e| e.to_string())?;
    Ok(socket)
}

fn serve_client(
    socket: &mut tungstenite::WebSocket<TcpStream>,
    id: &PlaygroundId,
    host: &PlaygroundHostService,
    outgoing: &mpsc::Receiver<PlaygroundEvent>,
    incoming: &mpsc::SyncSender<ClientMessage>,
    frames: &PlaygroundFrameQueue,
    stop: &AtomicBool,
) -> Result<(), String> {
    socket
        .send(Message::Text(
            serde_json::to_string(&host.connect(id)?)
                .map_err(|e| e.to_string())?
                .into(),
        ))
        .map_err(|e| e.to_string())?;
    pump_client(socket, outgoing, incoming, frames, stop)
}

fn pump_client(
    socket: &mut tungstenite::WebSocket<TcpStream>,
    outgoing: &mpsc::Receiver<PlaygroundEvent>,
    incoming: &mpsc::SyncSender<ClientMessage>,
    frames: &PlaygroundFrameQueue,
    stop: &AtomicBool,
) -> Result<(), String> {
    // Retain one decoded message when the host is busy. Stop reading more
    // input, but continue draining replies so neither direction deadlocks.
    let mut pending = None;
    while !stop.load(Ordering::Acquire) {
        if pending.is_none() {
            match socket.read() {
                Ok(Message::Text(text)) => {
                    let message: ClientMessage =
                        serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    pending = Some(message);
                }
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        if let Some(message) = pending.take() {
            match incoming.try_send(message) {
                Ok(()) => {}
                Err(mpsc::TrySendError::Full(message)) => pending = Some(message),
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    return Err("playground host input receiver disconnected".into());
                }
            }
        }
        for event in outgoing.try_iter().take(64) {
            socket
                .send(Message::Text(
                    serde_json::to_string(&event)
                        .map_err(|e| e.to_string())?
                        .into(),
                ))
                .map_err(|e| e.to_string())?;
        }
        if let Some(frame) = frames.take_latest() {
            socket
                .send(Message::Binary(frame.encode().into()))
                .map_err(|e| e.to_string())?;
        }
        if pending.is_some() {
            thread::sleep(Duration::from_millis(1));
        }
    }
    let _ = socket.close(None);
    Ok(())
}

pub fn read_playground_client_bootstrap() -> Result<PlaygroundClientBootstrap, String> {
    let mut input = std::io::stdin().lock();
    let mut length = [0u8; 4];
    input.read_exact(&mut length).map_err(|e| e.to_string())?;
    let length = u32::from_le_bytes(length) as usize;
    if length > 16 * 1024 {
        return Err("playground bootstrap exceeds size limit".into());
    }
    let mut bytes = vec![0; length];
    input.read_exact(&mut bytes).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

pub fn dispatch_playground_client() -> Option<amigo_core::AmigoResult<()>> {
    (std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("--runtime-playground-client")))
    .then(|| {
        let bootstrap =
            read_playground_client_bootstrap().map_err(amigo_core::AmigoError::Message)?;
        let assets = match bootstrap.client.as_str() {
            "npr-playground" => amigo_npr_playground_plugin::playground_client_assets(),
            other => {
                return Err(amigo_core::AmigoError::Message(format!(
                    "unregistered playground client: {other}"
                )));
            }
        };
        let native = bootstrap.native.clone();
        let mut web_bootstrap = serde_json::to_value(&bootstrap)
            .map_err(|e| amigo_core::AmigoError::Message(e.to_string()))?;
        web_bootstrap.as_object_mut().unwrap().remove("native");
        amigo_playground_tauri::run(
            bootstrap.label.clone(),
            web_bootstrap,
            native,
            assets,
        )
        .map_err(amigo_core::AmigoError::Message)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tungstenite::client::IntoClientRequest;

    #[test]
    fn playground_transport_backpressure_preserves_actions_replies_and_shutdown() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut client = tungstenite::WebSocket::from_raw_socket(
            stream, tungstenite::protocol::Role::Client, None,
        );
        let (stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_millis(5))).unwrap();
        stream.set_write_timeout(Some(Duration::from_millis(100))).unwrap();
        let (input_tx, input_rx) = mpsc::sync_channel(1);
        input_tx.send(ClientMessage::Refresh).unwrap();
        let (output_tx, output_rx) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let (done_tx, done_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut socket = tungstenite::WebSocket::from_raw_socket(
                stream, tungstenite::protocol::Role::Server, None,
            );
            let result = pump_client(&mut socket, &output_rx, &input_tx,
                &PlaygroundFrameQueue::default(), &stopped);
            done_tx.send(result).unwrap();
        });
        for request_id in 0..128 {
            client.send(Message::Text(serde_json::json!({
                "type":"action", "action": {
                    "request_id":request_id, "base_revision":0,
                    "control":"camera", "intent":null
                }
            }).to_string().into())).unwrap();
        }
        // The host has not consumed anything. A full input channel must not
        // terminate the connection or prevent a reply from reaching the UI.
        thread::sleep(Duration::from_millis(30));
        output_tx.send(PlaygroundEvent::Domain {
            name: "alive".into(), payload: serde_json::Value::Null,
        }).unwrap();
        let reply = client.read().unwrap();
        assert!(reply.to_text().unwrap().contains("alive"));
        assert!(matches!(input_rx.recv().unwrap(), ClientMessage::Refresh));
        for expected in 0..128 {
            let message = input_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(matches!(message, ClientMessage::Action { action } if action.request_id == expected));
        }
        // Closing the host must remain possible even while input is full.
        for _ in 0..3 {
            client.send(Message::Text("{\"type\":\"refresh\"}".into())).unwrap();
        }
        thread::sleep(Duration::from_millis(30));
        stop.store(true, Ordering::Release);
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap().unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn playground_websocket_checks_origin_token_and_accepts_authenticated_client() {
        for (origin, token, accepted) in [
            ("http://hostile.local", "secret", false),
            ("http://tauri.localhost", "wrong", false),
            ("http://tauri.localhost", "secret", true),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("ws://{}/", listener.local_addr().unwrap());
            let server = thread::spawn(move || {
                let (stream, _) = listener.accept().unwrap();
                let mut guard = PlaygroundHandshakeGuard::new(
                    PlaygroundId("fixture".into()),
                    "secret".into(),
                    "http://tauri.localhost".into(),
                );
                accept_client(stream, "http://tauri.localhost", &mut guard).is_ok()
            });
            let mut request = endpoint.into_client_request().unwrap();
            request
                .headers_mut()
                .insert("origin", origin.parse().unwrap());
            if let Ok((mut socket, _)) = tungstenite::connect(request) {
                socket
                    .send(Message::Text(
                        serde_json::to_string(&PlaygroundHandshake {
                            version: PLAYGROUND_PROTOCOL_VERSION,
                            playground: PlaygroundId("fixture".into()),
                            token: token.into(),
                        })
                        .unwrap()
                        .into(),
                    ))
                    .unwrap();
                assert_eq!(server.join().unwrap(), accepted);
            } else {
                assert!(!server.join().unwrap());
                assert!(!accepted);
            }
        }
    }

    #[test]
    fn playground_cube_has_no_companion_and_same_scene_is_not_reopened() {
        let companions = PlaygroundCompanionService::default();
        let host = Arc::new(PlaygroundHostService::default());
        companions.enable(PathBuf::from("must-never-be-executed"));
        companions
            .load_scene(Some("cube".into()), &[], host.clone())
            .unwrap();
        // Simulates a previously closed client: the scene identity is retained.
        let reference = amigo_scene::ScenePlaygroundReferenceDocument {
            id: "unregistered".into(),
            auto_open: true,
        };
        companions
            .load_scene(Some("cube".into()), &[reference], host.clone())
            .unwrap();
        assert!(companions.0.lock().unwrap().companions.is_empty());
        assert!(!companions.owns_primary_window());
        assert!(!companions.primary_window_closed());
        companions.load_scene(None, &[], host).unwrap();
        assert!(companions.0.lock().unwrap().scene.is_none());
    }
}
