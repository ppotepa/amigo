use super::*;
use std::{
    fs::File,
    io::{Read, Write},
    os::windows::io::{AsRawHandle, FromRawHandle},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use windows::{
    Win32::{Foundation::*, Storage::FileSystem::*, System::Pipes::*},
    core::PCWSTR,
};

pub struct NativeLink {
    outgoing: mpsc::SyncSender<NativeMessage>,
    incoming: Mutex<mpsc::Receiver<NativeMessage>>,
    stop: Arc<AtomicBool>,
    pub peer_pid: Arc<AtomicU32>,
}
impl Drop for NativeLink {
    fn drop(&mut self) {
        self.close();
    }
}
impl NativeLink {
    pub fn close(&self) {
        self.stop.store(true, Ordering::Release);
    }
    pub fn send(&self, message: NativeMessage) -> Result<(), String> {
        self.outgoing.try_send(message).map_err(|e| e.to_string())
    }
    pub fn drain(&self) -> Vec<NativeMessage> {
        self.incoming.lock().unwrap().try_iter().collect()
    }
    pub fn alive(&self) -> bool {
        !self.stop.load(Ordering::Acquire)
    }
}

fn channels() -> (
    Arc<NativeLink>,
    mpsc::Receiver<NativeMessage>,
    mpsc::SyncSender<NativeMessage>,
) {
    let (outgoing, send) = mpsc::sync_channel(16);
    let (receive, incoming) = mpsc::sync_channel(16);
    (
        Arc::new(NativeLink {
            outgoing,
            incoming: Mutex::new(incoming),
            stop: Arc::new(AtomicBool::new(false)),
            peer_pid: Arc::new(AtomicU32::new(0)),
        }),
        send,
        receive,
    )
}

pub fn native_server() -> Result<(NativeBootstrap, Arc<NativeLink>), String> {
    let mut entropy = [0u8; 32];
    getrandom::fill(&mut entropy).map_err(|e| e.to_string())?;
    let secret: String = entropy.iter().map(|byte| format!("{byte:02x}")).collect();
    let bootstrap = NativeBootstrap {
        pipe: format!(
            r"\\.\pipe\amigo-playground-{}-{}",
            std::process::id(),
            &secret[..16]
        ),
        secret,
        server_pid: std::process::id(),
    };
    let wide: Vec<u16> = bootstrap.pipe.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe {
        CreateNamedPipeW(
            PCWSTR(wide.as_ptr()),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_NOWAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            65536,
            65536,
            0,
            None,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(windows::core::Error::from_thread().to_string());
    }
    let file = unsafe { File::from_raw_handle(handle.0) };
    let (link, outgoing, incoming) = channels();
    let stop = link.stop.clone();
    let peer = link.peer_pid.clone();
    let token = bootstrap.secret.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let mut stream = Pipe {
                file,
                bytes: Vec::new(),
            };
            let start = Instant::now();
            loop {
                if stop.load(Ordering::Acquire) || start.elapsed() > Duration::from_secs(30) {
                    return Err("native handshake timeout".into());
                }
                let connected =
                    unsafe { ConnectNamedPipe(HANDLE(stream.file.as_raw_handle()), None) };
                if connected.is_ok()
                    || connected.as_ref().err().is_some_and(|e| {
                        e.code() == windows::core::HRESULT::from_win32(ERROR_PIPE_CONNECTED.0)
                    })
                {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            let mut pid = 0;
            unsafe { GetNamedPipeClientProcessId(HANDLE(stream.file.as_raw_handle()), &mut pid) }
                .map_err(|e| e.to_string())?;
            while peer.load(Ordering::Acquire) == 0
                && !stop.load(Ordering::Acquire)
                && start.elapsed() < Duration::from_secs(30)
            {
                std::thread::sleep(Duration::from_millis(1));
            }
            if pid != peer.load(Ordering::Acquire) {
                return Err("native companion PID mismatch".into());
            }
            loop {
                if stop.load(Ordering::Acquire) || start.elapsed() > Duration::from_secs(30) {
                    return Err("native handshake timeout".into());
                }
                if let Some(message) = stream.receive()? {
                    match message {
                        NativeMessage::Hello { secret } if authentic(&token, &secret) => break,
                        _ => return Err("native authentication rejected".into()),
                    }
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            stream.run(&outgoing, &incoming, &stop)
        })();
        if let Err(message) = result {
            let _ = incoming.try_send(NativeMessage::Error {
                generation: 0,
                message,
            });
        }
        stop.store(true, Ordering::Release);
    });
    Ok((bootstrap, link))
}

pub fn native_client(bootstrap: NativeBootstrap) -> Result<Arc<NativeLink>, String> {
    let wide: Vec<u16> = bootstrap.pipe.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
    .map_err(|e| e.to_string())?;
    let file = unsafe { File::from_raw_handle(handle.0) };
    let mut pid = 0;
    unsafe { GetNamedPipeServerProcessId(handle, &mut pid) }.map_err(|e| e.to_string())?;
    if pid != bootstrap.server_pid {
        return Err("native server PID mismatch".into());
    }
    let mode = PIPE_READMODE_BYTE | PIPE_NOWAIT;
    unsafe { SetNamedPipeHandleState(handle, Some(&mode), None, None) }
        .map_err(|e| e.to_string())?;
    let mut stream = Pipe {
        file,
        bytes: Vec::new(),
    };
    stream.send(&NativeMessage::Hello {
        secret: bootstrap.secret,
    })?;
    let (link, outgoing, incoming) = channels();
    link.peer_pid.store(pid, Ordering::Release);
    let stop = link.stop.clone();
    std::thread::spawn(move || {
        if let Err(message) = stream.run(&outgoing, &incoming, &stop) {
            let _ = incoming.try_send(NativeMessage::Error {
                generation: 0,
                message,
            });
        }
        stop.store(true, Ordering::Release);
    });
    Ok(link)
}

fn authentic(expected: &str, actual: &str) -> bool {
    expected.len() == actual.len()
        && !expected.is_empty()
        && expected
            .bytes()
            .zip(actual.bytes())
            .fold(0, |diff, (a, b)| diff | (a ^ b))
            == 0
}

struct Pipe {
    file: File,
    bytes: Vec<u8>,
}
impl Pipe {
    fn send(&mut self, message: &NativeMessage) -> Result<(), String> {
        let bytes = serde_json::to_vec(message).map_err(|e| e.to_string())?;
        if bytes.len() > 32768 {
            return Err("native message too large".into());
        }
        self.file
            .write_all(&(bytes.len() as u32).to_le_bytes())
            .and_then(|_| self.file.write_all(&bytes))
            .map_err(|e| e.to_string())
    }
    fn receive(&mut self) -> Result<Option<NativeMessage>, String> {
        let mut buffer = [0u8; 8192];
        match self.file.read(&mut buffer) {
            Ok(0) => {} // PIPE_NOWAIT byte pipes can succeed with no available bytes.
            Ok(count) => self.bytes.extend_from_slice(&buffer[..count]),
            Err(e) if matches!(e.raw_os_error(), Some(232) | Some(535) | Some(536)) => {}
            Err(e) => return Err(e.to_string()),
        }
        if self.bytes.len() < 4 {
            return Ok(None);
        }
        let length = u32::from_le_bytes(self.bytes[..4].try_into().unwrap()) as usize;
        if length > 32768 {
            return Err("native message too large".into());
        }
        if self.bytes.len() < length + 4 {
            return Ok(None);
        }
        let message =
            serde_json::from_slice(&self.bytes[4..length + 4]).map_err(|e| e.to_string())?;
        self.bytes.drain(..length + 4);
        Ok(Some(message))
    }
    fn run(
        &mut self,
        outgoing: &mpsc::Receiver<NativeMessage>,
        incoming: &mpsc::SyncSender<NativeMessage>,
        stop: &AtomicBool,
    ) -> Result<(), String> {
        while !stop.load(Ordering::Acquire) {
            for message in outgoing.try_iter() {
                self.send(&message)?;
            }
            if let Some(message) = self.receive()? {
                // The WebView/renderer may briefly fall behind while it is
                // negotiating resources or presenting a frame.  Do not turn
                // that transient backlog into a companion shutdown: preserve
                // message ordering and wait until the consumer makes room.
                let mut pending = message;
                loop {
                    match incoming.try_send(pending) {
                        Ok(()) => break,
                        Err(mpsc::TrySendError::Full(message)) => {
                            if stop.load(Ordering::Acquire) {
                                return Ok(());
                            }
                            pending = message;
                            std::thread::sleep(Duration::from_millis(1));
                            // Keep ownership of the message while waiting;
                            // control messages and ACKs must not be dropped.
                            continue;
                        }
                        Err(mpsc::TrySendError::Disconnected(_)) => {
                            return Err("native incoming channel disconnected".into());
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_secret_is_exact_and_nonempty() {
        assert!(authentic("abc", "abc"));
        assert!(!authentic("abc", "ab"));
        assert!(!authentic("abc", "abd"));
        assert!(!authentic("", ""));
    }
    #[test]
    fn authenticated_pipe_exchanges_metadata_and_closes() {
        let (bootstrap, server) = native_server().unwrap();
        server.peer_pid.store(std::process::id(), Ordering::Release);
        let client = native_client(bootstrap).unwrap();
        server
            .send(NativeMessage::RgbaConfigure {
                generation: 1,
                size: [16, 8],
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let messages = client.drain();
            for message in messages.iter().chain(server.drain().iter()) {
                if let NativeMessage::Error { message, .. } = message {
                    panic!("{message}");
                }
            }
            if messages.iter().any(|message| {
                matches!(message, NativeMessage::RgbaConfigure { generation: 1, .. })
            }) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "native channel did not authenticate"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        server.close();
        client.close();
    }
    #[test]
    fn incorrect_secret_never_delivers_native_resources() {
        let (mut bootstrap, server) = native_server().unwrap();
        server.peer_pid.store(std::process::id(), Ordering::Release);
        bootstrap.secret.push('x');
        let client = native_client(bootstrap).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if server.drain().iter().any(|message| matches!(message, NativeMessage::Error { message, .. } if message.contains("authentication rejected"))) { break; }
            assert!(
                Instant::now() < deadline,
                "invalid handshake was not rejected"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!server.alive());
        client.close();
    }
    #[test]
    fn unexpected_server_process_is_rejected_before_authentication() {
        let (mut bootstrap, server) = native_server().unwrap();
        bootstrap.server_pid = std::process::id().wrapping_add(1);
        assert!(
            matches!(native_client(bootstrap), Err(message) if message.contains("server PID mismatch"))
        );
        server.close();
    }
}
