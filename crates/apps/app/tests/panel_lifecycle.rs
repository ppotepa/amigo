use amigo_app::{BootstrapOptions, bootstrap_session_with_options};
use amigo_panels::PanelService;
use amigo_runtime::{Runtime, SystemPhase, SystemRegistry};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn runtime(mod_id: &str, scene: &str) -> Runtime {
    let mods = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../mods");
    bootstrap_session_with_options(
        BootstrapOptions::new(mods)
            .with_active_mods(vec!["core".into(), mod_id.into()])
            .with_startup_mod(mod_id)
            .with_startup_scene(scene)
            .with_dev_mode(true),
    )
    .unwrap()
    .into_parts()
    .0
    .into_runtime()
}
fn tick(runtime: &Runtime) {
    let systems = runtime.required::<SystemRegistry>().unwrap();
    for phase in [
        SystemPhase::PreUpdate,
        SystemPhase::FixedUpdate,
        SystemPhase::Update,
        SystemPhase::PostUpdate,
    ] {
        systems.run_phase(phase, runtime).unwrap();
    }
}

#[test]
fn headless_and_undeclared_scenes_do_not_spawn_panels() {
    let runtime = runtime("npr-playground", "gallery");
    tick(&runtime);
    let panels = runtime.required::<PanelService>().unwrap();
    assert!(
        panels
            .connection_snapshot("npr")
            .unwrap()
            .process_id
            .is_none()
    );
    let plain = self::runtime("playground-3d", "hello-world-cube");
    let panels = plain.required::<PanelService>().unwrap();
    panels.enable_host(PathBuf::from(env!("CARGO_BIN_EXE_amigo-app")));
    tick(&plain);
    assert!(panels.connection_snapshot("npr").is_none());
    assert!(panels.last_error().is_none());
}

#[test]
fn npr_zoom_consumes_scroll_once_per_host_frame_and_survives_pause_and_fit() {
    use amigo_runtime_control::{ControlValue, RuntimeControlService};
    let runtime = runtime("npr-playground", "gallery");
    tick(&runtime);
    let systems = runtime.required::<SystemRegistry>().unwrap();
    let controls = runtime.required::<RuntimeControlService>().unwrap();
    let input = runtime.required::<amigo_ui::UiInputService>().unwrap();
    let clock = runtime
        .required::<amigo_session::RuntimeFrameClockService>()
        .unwrap();
    let path = |name| format!("world.npr.settings.NprSettings.{name}");
    let distance = || {
        controls
            .get(&path("camera_distance"))
            .unwrap()
            .as_f64()
            .unwrap()
    };
    controls
        .set(&path("paused"), ControlValue::Bool(true))
        .unwrap();
    clock.force_single_simulation_tick(1.0 / 60.0);
    let before = distance();
    input.add_mouse_wheel(1.0);
    for _ in 0..4 {
        systems.run_phase(SystemPhase::Update, &runtime).unwrap();
    }
    assert_eq!(
        distance(),
        before,
        "simulation catch-up must not replay camera input"
    );
    systems
        .run_phase(SystemPhase::PostUpdate, &runtime)
        .unwrap();
    let first = distance();
    assert!(first < before && first > before * (-0.1_f64).exp());
    input.clear_frame_transients();
    for _ in 0..90 {
        systems
            .run_phase(SystemPhase::PostUpdate, &runtime)
            .unwrap();
    }
    assert!((distance() - before * (-0.1_f64).exp()).abs() < 0.0001);
    let settled = distance();
    input.add_mouse_wheel(-2.0);
    systems
        .run_phase(SystemPhase::PostUpdate, &runtime)
        .unwrap();
    assert!(distance() > settled);
    input.clear_frame_transients();
    controls
        .set(&path("fit"), ControlValue::Bool(true))
        .unwrap();
    let fitted = distance();
    for _ in 0..5 {
        systems
            .run_phase(SystemPhase::PostUpdate, &runtime)
            .unwrap();
    }
    assert_eq!(distance(), fitted, "fit cancels the old zoom target");
}

#[test]
fn workshop_authored_rhai_actions_change_metadata_and_support_undo() {
    use amigo_runtime_control::{ControlValue, RuntimeControlService};
    for scene in ["cube", "gallery"] {
        let runtime = runtime("npr-playground", scene);
        tick(&runtime);
        let scripts = runtime
            .required::<amigo_scripting_api::ScriptRuntimeService>()
            .unwrap();
        let controls = runtime.required::<RuntimeControlService>().unwrap();
        let source = format!("scene:npr-playground:{scene}");
        let p = "world.npr.settings.NprSettings.";
        scripts
            .call_on_event(&source, "npr.speed_double", &[])
            .unwrap();
        assert_eq!(
            controls.get(&format!("{p}speed")).unwrap(),
            ControlValue::F64(2.0)
        );
        scripts.call_on_event(&source, "npr.undo", &[]).unwrap();
        assert_eq!(
            controls.get(&format!("{p}speed")).unwrap(),
            ControlValue::F64(1.0)
        );
        scripts
            .call_on_event(&source, "npr.focus_selected", &[])
            .unwrap();
        assert_eq!(
            controls.get(&format!("{p}gallery")).unwrap(),
            ControlValue::Bool(false)
        );
        scripts
            .call_on_event(&source, "npr.layout_grid", &[])
            .unwrap();
        assert_eq!(
            controls.get(&format!("{p}gallery")).unwrap(),
            ControlValue::Bool(true)
        );
        scripts
            .call_on_event(&source, "npr.capture_before", &[])
            .unwrap();
        assert_eq!(
            controls.get(&format!("{p}can_compare")).unwrap(),
            ControlValue::Bool(true)
        );
    }
}

// Native-window lifecycle acceptance runs on the project's Windows host. The
// protocol entrypoint tests remain display-independent on all platforms.
#[cfg(windows)]
#[test]
fn native_panel_reopens_through_rhai_and_unloads_cleanly() {
    let runtime = runtime("npr-playground", "gallery");
    let panels = runtime.required::<PanelService>().unwrap();
    panels.enable_host(PathBuf::from(env!("CARGO_BIN_EXE_amigo-app")));
    let wait_ready = || {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            tick(&runtime);
            if let Some(s) = panels.connection_snapshot("npr") {
                assert!(s.failure.is_none(), "{:?}", s.failure);
                if s.ready {
                    return s;
                }
            }
            assert!(
                Instant::now() < deadline,
                "panel not ready: {:?}; state={:?}; console={:?}; pending={:?}",
                panels.last_error(),
                panels.connection_snapshot("npr"),
                runtime
                    .required::<amigo_scripting_api::DevConsoleState>()
                    .unwrap()
                    .output_tail(5),
                runtime
                    .required::<amigo_scripting_api::DevConsoleQueue>()
                    .unwrap()
                    .pending()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    let first = wait_ready();
    panels.close("npr").unwrap();
    tick(&runtime);
    assert!(
        panels
            .connection_snapshot("npr")
            .unwrap()
            .process_id
            .is_none()
    );
    runtime
        .required::<amigo_scripting_api::ScriptRuntimeService>()
        .unwrap()
        .eval_console(
            amigo_scripting_api::DevConsoleScriptContext::new(Some("gallery".into())),
            "world.panels.open(\"npr\");",
        )
        .unwrap();
    let second = wait_ready();
    assert_ne!(first.process_id, second.process_id);
    panels
        .load_scene(None, Path::new("."), Path::new("."))
        .unwrap();
    assert!(panels.connection_snapshot("npr").is_none());
}
