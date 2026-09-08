// Shared executable-level regression: client mode must work outside a project
// directory and its first stdout bytes must be protocol, never TUI/log output.
pub fn verify(executable: &str) {
    use std::{
        io::Read,
        process::{Command, Stdio},
        sync::mpsc,
        time::Duration,
    };
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut command = Command::new(executable);
    command
        .arg("--runtime-panel-client")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = ChildGuard(command.spawn().unwrap());
    let mut stdout = child.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let result = (|| -> std::io::Result<String> {
            let mut header = [0; 4];
            stdout.read_exact(&mut header)?;
            let size = u32::from_le_bytes(header) as usize;
            if size > 1024 {
                return Err(std::io::Error::other(
                    "non-protocol stdout before handshake",
                ));
            }
            let mut bytes = vec![0; size];
            stdout.read_exact(&mut bytes)?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        })();
        let _ = tx.send(result);
    });
    let response = rx.recv_timeout(Duration::from_secs(5));
    drop(child);
    reader.join().unwrap();
    assert_eq!(
        response
            .expect("client did not handshake before normal startup")
            .expect("invalid client stdout"),
        "{\"Hello\":{\"version\":1}}"
    );
}
