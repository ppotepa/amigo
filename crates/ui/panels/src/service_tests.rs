use super::*;
use amigo_runtime_control::*;

fn fixture_command(version: Option<u32>, exit: bool) -> Command {
    #[cfg(windows)]
    {
        let mut command = Command::new("powershell.exe");
        let send=version.map(|v|format!("$b=[Text.Encoding]::UTF8.GetBytes('{{\"Hello\":{{\"version\":{v}}}}}');$o=[Console]::OpenStandardOutput();$h=[BitConverter]::GetBytes([int]$b.Length);$o.Write($h,0,4);$o.Write($b,0,$b.Length);$o.Flush();")).unwrap_or_default();
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "{send}{}",
                if exit {
                    "exit 7"
                } else {
                    "Start-Sleep -Seconds 60"
                }
            ),
        ]);
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
        command
    }
    #[cfg(not(windows))]
    {
        let mut command = Command::new("sh");
        let send = version
            .map(|v| {
                let body = format!("{{\"Hello\":{{\"version\":{v}}}}}");
                format!(
                    "printf '\\{:03o}\\000\\000\\000'; printf '%s' '{body}';",
                    body.len()
                )
            })
            .unwrap_or_default();
        command.args([
            "-c",
            &format!("{send}{}", if exit { "exit 7" } else { "exec sleep 60" }),
        ]);
        command
    }
}
fn fixture_service(version: Option<u32>, exit: bool) -> PanelService {
    let service = PanelService::default();
    let mut panel = Panel {
        path: PathBuf::new(),
        source: String::new(),
        document: layout("value"),
        revision: 1,
        error: None,
        failure: None,
        connection: None,
    };
    let mut command = fixture_command(version, exit);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    panel.connection = Some(spawn_connection(&mut command, 0, &panel).unwrap());
    service
        .state
        .lock()
        .unwrap()
        .panels
        .insert("test".into(), panel);
    service
}
fn poll_until(service: &PanelService, condition: impl Fn(&Panel) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        service.tick(
            &controls(),
            &ScriptEventQueue::default(),
            &crate::PresetService::default(),
        );
        if condition(&service.state.lock().unwrap().panels["test"]) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "panel did not reach expected state: {:?}",
            service.last_error()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn real_process_handshake_close_and_scene_cleanup() {
    let service = fixture_service(Some(PROTOCOL_VERSION), false);
    poll_until(&service, |p| p.connection.as_ref().is_some_and(|c| c.ready));
    assert!(service.last_error().is_none());
    service.close("test").unwrap();
    assert!(
        service.state.lock().unwrap().panels["test"]
            .connection
            .is_none()
    );
    // Reopening installs a fresh transport; no prior request/ready state survives.
    {
        let mut state = service.state.lock().unwrap();
        let panel = state.panels.get_mut("test").unwrap();
        let mut command = fixture_command(Some(PROTOCOL_VERSION), false);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        panel.connection = Some(spawn_connection(&mut command, 0, panel).unwrap());
    }
    poll_until(&service, |p| p.connection.as_ref().is_some_and(|c| c.ready));
    service.state.lock().unwrap().scene = Some("old".into());
    service
        .load_scene(None, Path::new("."), Path::new("."))
        .unwrap();
    assert!(service.state.lock().unwrap().panels.is_empty());
}
#[test]
fn real_process_bad_protocol_exit_crash_and_timeout_are_persistent_failures() {
    for (version, exit, expected) in [
        (Some(999), false, "unsupported panel protocol"),
        (None, true, "panel"),
        (None, false, "timed out"),
    ] {
        let service = fixture_service(version, exit);
        if version.is_none() && !exit {
            service
                .state
                .lock()
                .unwrap()
                .panels
                .get_mut("test")
                .unwrap()
                .connection
                .as_mut()
                .unwrap()
                .started -= Duration::from_secs(6);
        }
        poll_until(&service, |p| p.connection.is_none());
        assert!(service.last_error().unwrap().contains(expected));
        let before = service.last_error();
        service.tick(
            &controls(),
            &ScriptEventQueue::default(),
            &crate::PresetService::default(),
        );
        assert_eq!(service.last_error(), before);
    }
    let service = fixture_service(Some(PROTOCOL_VERSION), false);
    poll_until(&service, |p| p.connection.as_ref().is_some_and(|c| c.ready));
    service
        .state
        .lock()
        .unwrap()
        .panels
        .get_mut("test")
        .unwrap()
        .connection
        .as_mut()
        .unwrap()
        .child
        .kill()
        .unwrap();
    poll_until(&service, |p| p.connection.is_none());
    assert!(service.last_error().is_some());
}
#[test]
fn spawn_failure_does_not_retry_on_each_frame_or_block_scene() {
    let root = temp();
    let scene = root.join("scene.yml");
    std::fs::write(
        root.join("panel.yml"),
        serde_yaml::to_string(&layout("value")).unwrap(),
    )
    .unwrap();
    std::fs::write(
        &scene,
        "panels: [{id: test, layout: panel.yml, auto_open: true}]",
    )
    .unwrap();
    let service = PanelService::default();
    service.enable_host(root.join("missing-host"));
    service
        .load_scene(Some("scene".into()), &root, &scene)
        .unwrap();
    service.tick(
        &controls(),
        &ScriptEventQueue::default(),
        &crate::PresetService::default(),
    );
    assert!(service.last_error().unwrap().contains("could not start"));
    let generation = service.state.lock().unwrap().generation;
    service
        .load_scene(Some("scene".into()), &root, &scene)
        .unwrap();
    assert_eq!(service.state.lock().unwrap().generation, generation);
    assert!(service.open("test").is_err());
    std::fs::remove_dir_all(root).unwrap();
}
struct Provider(Mutex<f64>);
impl RuntimeControlProvider for Provider {
    fn provider_id(&self) -> &'static str {
        "test"
    }
    fn rebuild_registry(&self, r: &mut RuntimeControlRegistry) -> Result<(), RuntimeControlError> {
        for (path, writable) in [("value", true), ("readonly", false)] {
            r.register_property(RuntimeControlProperty {
                console_path: path.into(),
                target_path: "test".into(),
                component: None,
                property_path: path.into(),
                value_type: ControlValueType::F64,
                range: Some(ControlRange {
                    min: Some(0.0),
                    max: Some(1.0),
                }),
                writable,
                readable: true,
                animatable: false,
                source_file: None,
                source_pointer: None,
                provider_id: "test".into(),
                description: None,
            });
        }
        Ok(())
    }
    fn get(&self, _: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        Ok(ControlValue::F64(*self.0.lock().unwrap()))
    }
    fn set(&self, _: &RuntimeControlProperty, v: ControlValue) -> Result<(), RuntimeControlError> {
        *self.0.lock().unwrap() = v.as_f64().unwrap();
        Ok(())
    }
}
fn layout(binding: &str) -> PanelDocument {
    serde_yaml::from_str(&format!("id: test\ntitle: Test\nroot:\n  type: slider\n  id: edit\n  min: 0.0\n  max: 1.0\n  value_bind: {binding}\n")).unwrap()
}
fn controls() -> RuntimeControlService {
    let s = RuntimeControlService::default();
    s.register_provider(Arc::new(Provider(Mutex::new(0.5))));
    s
}

#[test]
fn host_diagnostics_are_visible_without_a_panel_and_not_spammed() {
    let root = temp();
    let runtime = amigo_runtime::RuntimeBuilder::default()
        .with_service(amigo_scripting_api::DevConsoleState::default())
        .unwrap()
        .with_service(amigo_scripting_api::RunLogService::new(&root).unwrap())
        .unwrap()
        .build();
    let service = PanelService::default();
    service.state.lock().unwrap().error = Some("panel test handshake timed out".into());
    service.report_diagnostics(&runtime);
    service.report_diagnostics(&runtime);
    let console = runtime
        .required::<amigo_scripting_api::DevConsoleState>()
        .unwrap();
    assert_eq!(
        console
            .output_lines()
            .iter()
            .filter(|line| line.contains("[panels.host]"))
            .count(),
        1
    );
    let log = runtime
        .required::<amigo_scripting_api::RunLogService>()
        .unwrap();
    assert!(
        std::fs::read_to_string(log.runtime_log_path())
            .unwrap()
            .contains("handshake timed out")
    );
    drop(log);
    drop(runtime);
    std::fs::remove_dir_all(root).unwrap();
}
fn temp() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "amigo-panel-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn stale_readonly_invalid_and_unknown_edits_do_not_mutate() {
    let c = controls();
    let events = ScriptEventQueue::default();
    let doc = layout("value");
    assert!(
        apply_edit(
            &doc,
            2,
            3,
            1,
            3,
            "edit",
            ControlValue::F64(0.9),
            &c,
            &events
        )
        .is_err()
    );
    assert!(
        apply_edit(
            &doc,
            2,
            3,
            2,
            2,
            "edit",
            ControlValue::F64(0.9),
            &c,
            &events
        )
        .is_err()
    );
    assert!(
        apply_edit(
            &layout("readonly"),
            2,
            3,
            2,
            3,
            "edit",
            ControlValue::F64(0.9),
            &c,
            &events
        )
        .is_err()
    );
    assert!(
        apply_edit(
            &doc,
            2,
            3,
            2,
            3,
            "edit",
            ControlValue::F64(2.0),
            &c,
            &events
        )
        .is_err()
    );
    assert!(
        layout("missing")
            .validate_bindings(&c.registry_snapshot())
            .is_err()
    );
    assert_eq!(c.get("value").unwrap(), ControlValue::F64(0.5));
    apply_edit(
        &doc,
        2,
        3,
        2,
        3,
        "edit",
        ControlValue::F64(0.9),
        &c,
        &events,
    )
    .unwrap();
    assert_eq!(c.get("value").unwrap(), ControlValue::F64(0.9));
}
#[test]
fn hot_reload_preserves_state_and_last_valid_layout() {
    let root = temp();
    let path = root.join("panel.yml");
    let scene = root.join("scene.yml");
    std::fs::write(&path, serde_yaml::to_string(&layout("value")).unwrap()).unwrap();
    std::fs::write(
        &scene,
        "panels: [{id: test, layout: panel.yml, auto_open: false}]",
    )
    .unwrap();
    let panels = PanelService::default();
    panels
        .load_scene(Some("first".into()), &root, &scene)
        .unwrap();
    let c = controls();
    c.set("value", ControlValue::F64(0.8)).unwrap();
    let events = ScriptEventQueue::default();
    let presets = crate::PresetService::default();
    std::fs::write(&path, "invalid: [").unwrap();
    panels.state.lock().unwrap().last_poll = Instant::now() - Duration::from_secs(1);
    panels.tick(&c, &events, &presets);
    assert_eq!(panels.state.lock().unwrap().panels["test"].revision, 1);
    assert!(panels.last_error().is_some());
    let mut changed = layout("value");
    changed.title = "Reloaded".into();
    std::fs::write(&path, serde_yaml::to_string(&changed).unwrap()).unwrap();
    panels.state.lock().unwrap().last_poll = Instant::now() - Duration::from_secs(1);
    panels.tick(&c, &events, &presets);
    assert_eq!(panels.state.lock().unwrap().panels["test"].revision, 2);
    assert_eq!(c.get("value").unwrap(), ControlValue::F64(0.8));
    panels.load_scene(None, &root, &scene).unwrap();
    assert!(panels.state.lock().unwrap().panels.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}
