use super::super::*;
use amigo_app_host_api::HostControl;

fn console_test_host() -> InteractiveRuntimeHostHandler {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
            .with_startup_mod("core-game")
            .with_startup_scene("console")
            .with_dev_mode(true),
    )
    .expect("console bootstrap should succeed");

    InteractiveRuntimeHostHandler::new(amigo_session::RuntimeSession::from_runtime(runtime, amigo_session::RuntimeSessionProfile::Game), summary)
        .expect("interactive host handler should initialize")
}

#[test]
fn dev_console_accepts_text_input_when_open() {
    let mut host = console_test_host();

    host.on_input_event(InputEvent::Key {
        key: KeyCode::Backquote,
        pressed: true,
    })
    .expect("console toggle should be accepted");
    host.on_input_event(InputEvent::TextInput {
        text: "render.stats".to_owned(),
    })
    .expect("console text input should be accepted");

    let console = host
        .session.runtime()
        .resolve::<DevConsoleState>()
        .expect("dev console state should exist");
    assert_eq!(console.input(), "render.stats");
}

#[test]
fn dev_console_enter_submits_command() {
    let mut host = console_test_host();

    host.on_input_event(InputEvent::Key {
        key: KeyCode::Backquote,
        pressed: true,
    })
    .expect("console toggle should be accepted");
    host.on_input_event(InputEvent::TextInput {
        text: "echo hello".to_owned(),
    })
    .expect("console text input should be accepted");
    host.on_input_event(InputEvent::Key {
        key: KeyCode::Enter,
        pressed: true,
    })
    .expect("console enter should be accepted");

    let queue = host
        .session.runtime()
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist");
    assert_eq!(queue.pending()[0].line, "echo hello");
}

#[test]
fn dev_console_escape_closes_without_exit() {
    let mut host = console_test_host();

    host.on_input_event(InputEvent::Key {
        key: KeyCode::Backquote,
        pressed: true,
    })
    .expect("console toggle should be accepted");
    let outcome = host
        .on_input_event(InputEvent::Key {
            key: KeyCode::Escape,
            pressed: true,
        })
        .expect("console escape should be accepted");

    let console = host
        .session.runtime()
        .resolve::<DevConsoleState>()
        .expect("dev console state should exist");
    assert!(matches!(outcome, HostControl::Continue));
    assert!(!console.is_open());
}

#[test]
fn dev_console_mouse_wheel_scrolls_output_when_open() {
    let mut host = console_test_host();
    let console = host
        .session.runtime()
        .resolve::<DevConsoleState>()
        .expect("dev console state should exist");
    for index in 0..20 {
        console.write_line(format!("line-{index}"));
    }

    host.on_input_event(InputEvent::Key {
        key: KeyCode::Backquote,
        pressed: true,
    })
    .expect("console toggle should be accepted");
    host.on_input_event(InputEvent::MouseWheel { delta_y: 120.0 })
        .expect("console wheel should be accepted");

    assert!(console.output_scroll_offset() > 0);
}




