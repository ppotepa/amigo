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
        host: PanelHost::External,
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
    assert!(service.state.lock().unwrap().panels["test"]
        .connection
        .is_none());
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

#[test]
fn embedded_panel_is_available_as_snapshot_without_starting_an_external_process() {
    let root = temp();
    let scene = root.join("scene.yml");
    std::fs::write(
        root.join("panel.yml"),
        serde_yaml::to_string(&layout("value")).unwrap(),
    )
    .unwrap();
    std::fs::write(
        &scene,
        "panels: [{id: test, layout: panel.yml, host: embedded, auto_open: true}]",
    )
    .unwrap();
    let service = PanelService::default();
    service.enable_host(root.join("missing-host"));
    service
        .load_scene(Some("scene".into()), &root, &scene)
        .unwrap();
    let controls = controls();
    let events = ScriptEventQueue::default();
    let presets = crate::PresetService::default();
    service.tick(&controls, &events, &presets);
    assert!(service.last_error().is_none());
    assert!(service
        .connection_snapshot("test")
        .unwrap()
        .process_id
        .is_none());
    assert_eq!(service.snapshots(&controls, &presets).unwrap().len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn both_panel_keeps_both_host_capabilities_in_one_authored_entry() {
    let root = temp();
    let scene = root.join("scene.yml");
    std::fs::write(
        root.join("panel.yml"),
        serde_yaml::to_string(&layout("value")).unwrap(),
    )
    .unwrap();
    std::fs::write(
        &scene,
        "panels: [{id: test, layout: panel.yml, host: both, auto_open: false}]",
    )
    .unwrap();
    let service = PanelService::default();
    service
        .load_scene(Some("scene".into()), &root, &scene)
        .unwrap();
    let controls = controls();
    let presets = crate::PresetService::default();
    assert_eq!(service.snapshots(&controls, &presets).unwrap().len(), 1);
    let host = service.state.lock().unwrap().panels["test"].host;
    assert!(host.includes_embedded());
    assert!(host.includes_external());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn embedded_snapshot_builds_a_menu_overlay_with_the_authored_control_tree() {
    let service = PanelService::default();
    service.state.lock().unwrap().panels.insert(
        "test".into(),
        Panel {
            failure: None,
            path: PathBuf::new(),
            source: String::new(),
            document: layout("value"),
            revision: 1,
            error: None,
            host: PanelHost::Embedded,
            connection: None,
        },
    );
    let overlays = service
        .embedded_overlays(&controls(), &crate::PresetService::default())
        .unwrap();
    assert_eq!(overlays.len(), 1);
    assert_eq!(overlays[0].entity_name, "panel:test");
    assert_eq!(
        overlays[0].root.children[1].children[0].id.as_deref(),
        Some("edit")
    );
}

#[test]
fn embedded_panel_scroll_viewport_fills_the_available_window_height() {
    let snapshot = amigo_panel_api::PanelSnapshot {
        generation: 1,
        revision: 1,
        document: layout("value"),
        preset_names: vec![],
        values: Default::default(),
    };
    let overlay = crate::overlay(
        &snapshot,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    let layout = build_ui_layout_tree(
        amigo_overlay_api::UiViewportSize::new(1024.0, 600.0),
        &overlay,
    );
    let scroll = &layout.children[1];

    assert_eq!(layout.rect.y, 20.0);
    assert_eq!(layout.rect.height, 560.0);
    assert!(scroll.rect.height > 0.0);
    assert!(scroll.rect.y + scroll.rect.height <= layout.rect.y + layout.rect.height - 14.0);
}

#[test]
fn dropdown_selection_cycles_only_declared_values() {
    assert_eq!(
        next_option(&["cube".into(), "sphere".into()], "cube"),
        Some("sphere".into())
    );
    assert_eq!(
        next_option(&["cube".into(), "sphere".into()], "sphere"),
        Some("cube".into())
    );
    assert_eq!(next_option(&[], "cube"), None);
}

#[test]
fn embedded_option_cards_resolve_to_their_exact_authored_value() {
    let document: PanelDocument = serde_yaml::from_str(
        "id: test\ntitle: Test\npresentation:\n  style:\n    choices:\n      - {value: Comic Ink, label: Comic Ink, artwork: ink}\n      - {value: Pencil Study, label: Pencil Study}\nartwork:\n  ink:\n    - {points: [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]], color: [12, 34, 56]}\nroot:\n  type: option-set\n  id: style\n  options: [Comic Ink, Pencil Study]\n",
    )
    .unwrap();
    let snapshot = amigo_panel_api::PanelSnapshot {
        generation: 1,
        revision: 1,
        document,
        preset_names: vec![],
        values: Default::default(),
    };
    let card = crate::choice_id("test", "style", "Pencil Study");
    assert_eq!(
        crate::choice_value_for_id(&snapshot, &card),
        Some(("style".into(), "Pencil Study".into()))
    );
    let overlay = crate::overlay(
        &snapshot,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    let rows = &overlay.root.children[1].children[0].children;
    assert_eq!(rows.len(), 1);
    let cards = &rows[0].children;
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[1].id.as_deref(), Some(card.as_str()));
    assert_eq!(cards[0].style.preview_triangles.len(), 1);
    assert!(cards[1].style.preview_triangles.is_empty());
    let hovered = crate::overlay(
        &snapshot,
        &Default::default(),
        &Default::default(),
        &[("test".to_owned(), card.clone())].into_iter().collect(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    let hovered_card = &hovered.root.children[1].children[0].children[0].children[1];
    assert_ne!(hovered_card.style.background, cards[1].style.background);
}

#[test]
fn option_set_without_choice_presentation_remains_compact() {
    let document: PanelDocument = serde_yaml::from_str(
        "id: test\ntitle: Test\nroot:\n  type: option-set\n  id: blend\n  options: [normal, multiply]\n",
    )
    .unwrap();
    let snapshot = amigo_panel_api::PanelSnapshot {
        generation: 1,
        revision: 1,
        document,
        preset_names: vec![],
        values: Default::default(),
    };
    let overlay = crate::overlay(
        &snapshot,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    assert!(matches!(
        overlay.root.children[1].children[0].kind,
        amigo_overlay_api::UiOverlayNodeKind::OptionSet { .. }
    ));
}

#[test]
fn embedded_group_box_uses_authored_default_and_local_expansion_state() {
    let document: PanelDocument = serde_yaml::from_str(
        "id: test\ntitle: Test\npresentation:\n  advanced: {collapsed: true}\nroot:\n  type: group-box\n  id: advanced\n  text: Advanced\n  children:\n    - {type: text, id: detail, text: Detail}\n",
    )
    .unwrap();
    let snapshot = amigo_panel_api::PanelSnapshot {
        generation: 1,
        revision: 1,
        document,
        preset_names: vec![],
        values: Default::default(),
    };
    let collapsed = crate::overlay(
        &snapshot,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    assert!(collapsed.root.children[1].children[0].children.is_empty());
    let expanded = crate::overlay(
        &snapshot,
        &Default::default(),
        &[("test:advanced".to_owned(), false)].into_iter().collect(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    assert_eq!(expanded.root.children[1].children[0].children.len(), 1);
}

#[test]
fn embedded_dropdown_expands_and_resolves_the_clicked_option() {
    let document: PanelDocument = serde_yaml::from_str(
        "id: test\ntitle: Test\nroot:\n  type: dropdown\n  id: model\n  options: [cube, sphere, suzanne]\n",
    )
    .unwrap();
    let snapshot = amigo_panel_api::PanelSnapshot {
        generation: 1,
        revision: 1,
        document,
        preset_names: vec![],
        values: Default::default(),
    };
    let overlay = crate::overlay(
        &snapshot,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &[("test:model".to_owned(), true)].into_iter().collect(),
        &Default::default(),
        &Default::default(),
    );
    let node = &overlay.root.children[1].children[0];
    assert!(matches!(
        &node.kind,
        amigo_overlay_api::UiOverlayNodeKind::Dropdown { expanded: true, .. }
    ));
    assert!(expanded_dropdown_owns_wheel(&node.kind));
    let closed = crate::overlay(
        &snapshot,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    assert!(!expanded_dropdown_owns_wheel(
        &closed.root.children[1].children[0].kind
    ));
    assert_eq!(
        dropdown_value_from_mouse(
            amigo_overlay_api::UiRect::new(0.0, 0.0, 200.0, 38.0),
            &["cube".into(), "sphere".into(), "suzanne".into(),],
            0.0,
            39.0
        ),
        Some("cube".into())
    );
}

#[test]
fn embedded_dropdown_scroll_offset_is_clamped_to_its_visible_range() {
    let service = PanelService::default();
    service.set_embedded_dropdown_scroll("test", "model", 99.0, 14);
    assert_eq!(
        service.state.lock().unwrap().embedded_dropdown_scrolls["test:model"],
        4.0
    );
}

#[test]
fn embedded_panel_scroll_offset_is_clamped_to_content_range() {
    let service = PanelService::default();
    service.set_embedded_scroll_offset("test", 999.0, 240.0);
    assert_eq!(
        service.state.lock().unwrap().embedded_scroll_offsets["test"],
        240.0
    );
    service.set_embedded_scroll_offset("test", -12.0, 240.0);
    assert_eq!(
        service.state.lock().unwrap().embedded_scroll_offsets["test"],
        0.0
    );
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
    assert!(std::fs::read_to_string(log.runtime_log_path())
        .unwrap()
        .contains("handshake timed out"));
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
    assert!(apply_edit(
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
    .is_err());
    assert!(apply_edit(
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
    .is_err());
    assert!(apply_edit(
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
    .is_err());
    assert!(apply_edit(
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
    .is_err());
    assert!(layout("missing")
        .validate_bindings(&c.registry_snapshot())
        .is_err());
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
fn transport_independent_snapshot_and_interaction_share_panel_validation() {
    let service = PanelService::default();
    service.state.lock().unwrap().panels.insert(
        "test".into(),
        Panel {
            failure: None,
            path: PathBuf::new(),
            source: String::new(),
            document: layout("value"),
            revision: 3,
            error: None,
            host: PanelHost::Embedded,
            connection: None,
        },
    );
    let controls = controls();
    let events = ScriptEventQueue::default();
    let snapshot = service
        .snapshots(&controls, &crate::PresetService::default())
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(snapshot.document.id, "test");
    assert_eq!(snapshot.values["value"].value, ControlValue::F64(0.5));

    service
        .apply_interaction(
            PanelInteraction::Edit {
                panel_id: "test".into(),
                generation: snapshot.generation,
                revision: snapshot.revision,
                control: "edit".into(),
                value: ControlValue::F64(0.8),
            },
            &controls,
            &events,
        )
        .unwrap();
    assert_eq!(controls.get("value").unwrap(), ControlValue::F64(0.8));
    assert!(service
        .apply_interaction(
            PanelInteraction::Edit {
                panel_id: "test".into(),
                generation: snapshot.generation,
                revision: snapshot.revision + 1,
                control: "edit".into(),
                value: ControlValue::F64(0.2),
            },
            &controls,
            &events,
        )
        .is_err());
    assert_eq!(controls.get("value").unwrap(), ControlValue::F64(0.8));
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
