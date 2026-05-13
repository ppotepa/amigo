use crate::{
    DevConsoleCommand, DevConsoleOutputLevel, DevConsoleQueue, DevConsoleState, ScriptCommand,
    ScriptCommandQueue, ScriptEvent, ScriptEventQueue, RunLogService,
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
        vec![
            "> help".to_owned(),
            "available placeholder commands: help".to_owned()
        ]
    );
}

#[test]
fn dev_console_splits_multiline_output_and_tracks_levels() {
    let state = DevConsoleState::default();

    state.write_line_with_level("alpha\nbeta", DevConsoleOutputLevel::Warning);

    let entries = state.output_entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].text, "alpha");
    assert_eq!(entries[1].text, "beta");
    assert_eq!(entries[0].level, DevConsoleOutputLevel::Warning);
}

#[test]
fn run_log_service_splits_runtime_and_console_logs() {
    let directory = std::env::temp_dir().join(format!(
        "amigo-run-log-test-{}",
        std::process::id()
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }

    let run_log = RunLogService::new_with_run_id(&directory, "abcd").unwrap();
    assert_eq!(run_log.runtime_log_path(), directory.join("abcd.runtime.log"));
    assert_eq!(run_log.console_log_path(), directory.join("abcd.console.log"));

    run_log.write_runtime("runtime boot");
    run_log.write_console("console verbose command trace");

    let runtime_log = std::fs::read_to_string(run_log.runtime_log_path()).unwrap();
    let console_log = std::fs::read_to_string(run_log.console_log_path()).unwrap();
    assert!(runtime_log.contains("runtime boot"));
    assert!(console_log.contains("console verbose command trace"));

    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn dev_console_writes_attached_console_log() {
    let directory = std::env::temp_dir().join(format!(
        "amigo-console-log-test-{}",
        std::process::id()
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }

    let run_log = std::sync::Arc::new(RunLogService::new_with_run_id(&directory, "efgh").unwrap());
    let state = DevConsoleState::default();
    state.attach_run_log(run_log.clone());
    state.record_command("stats");
    state.write_line_with_level("scene.active=main", DevConsoleOutputLevel::Success);

    let console_log = std::fs::read_to_string(run_log.console_log_path()).unwrap();
    assert!(console_log.contains("input raw=\"stats\""));
    assert!(console_log.contains("output level=Success"));

    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn dev_console_input_editor_moves_cursor_and_inserts() {
    let state = DevConsoleState::default();

    state.set_input_with_cursor("postfx.items ad blur", "postfx.items ad".len());
    state.insert_input_text("d");

    assert_eq!(state.input(), "postfx.items add blur");
}

#[test]
fn dev_console_input_editor_selects_and_replaces() {
    let state = DevConsoleState::default();

    state.set_input("opacity");
    state.move_input_home(false);
    state.move_input_right(true, false);
    state.move_input_right(true, false);
    state.insert_input_text("vi");

    assert_eq!(state.input(), "viacity");
}

#[test]
fn dev_console_internal_clipboard_cuts_and_pastes() {
    let state = DevConsoleState::default();

    state.set_input("scene stats");
    state.select_all_input();

    assert!(state.cut_input_selection());
    assert_eq!(state.input(), "");

    assert!(state.paste_input_clipboard());
    assert_eq!(state.input(), "scene stats");
}

#[test]
fn dev_console_output_window_respects_scroll_offset() {
    let state = DevConsoleState::default();
    for index in 0..6 {
        state.write_line(format!("line-{index}"));
    }

    assert_eq!(
        state
            .output_window(3)
            .into_iter()
            .map(|line| line.text)
            .collect::<Vec<_>>(),
        vec!["line-3", "line-4", "line-5"]
    );

    state.scroll_output(2);

    assert_eq!(state.output_scroll_offset(), 2);
    assert_eq!(
        state
            .output_window(3)
            .into_iter()
            .map(|line| line.text)
            .collect::<Vec<_>>(),
        vec!["line-1", "line-2", "line-3"]
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
            vec!["playground-2d-ui-preview.root.control-card.button-row.repair-button".to_owned()],
        )
    );
    assert_eq!(
        ScriptCommand::ui_disable(
            "playground-2d-ui-preview.root.control-card.button-row.repair-button"
        ),
        ScriptCommand::new(
            "ui",
            "disable",
            vec!["playground-2d-ui-preview.root.control-card.button-row.repair-button".to_owned()],
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

