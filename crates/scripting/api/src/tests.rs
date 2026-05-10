use crate::{
    DevConsoleCommand, DevConsoleOutputLevel, DevConsoleQueue, DevConsoleState, ScriptCommand,
    ScriptCommandQueue, ScriptEvent, ScriptEventQueue,
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
