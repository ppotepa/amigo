use amigo_app::{BootstrapOptions, bootstrap_session_with_options};
use amigo_npr_playground_plugin::{
    NprPlaygroundState,
    playground::{NprPlaygroundIntent, NprPlaygroundService},
};
use amigo_runtime::{Runtime, SystemPhase, SystemRegistry};
use amigo_runtime_bundles::PlaygroundCompanionService;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn runtime(scene: &str) -> Runtime {
    let mods = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../mods");
    bootstrap_session_with_options(
        BootstrapOptions::new(mods)
            .with_active_mods(vec!["core".into(), "npr-playground".into()])
            .with_startup_mod("npr-playground")
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
fn edit(service: &NprPlaygroundService, intent: NprPlaygroundIntent) {
    service
        .dispatch_intent(1, service.domain_snapshot().revision, "test".into(), intent)
        .unwrap();
}
fn navigate(mode: &str, wheel: f32) -> NprPlaygroundIntent {
    NprPlaygroundIntent::Navigate {
        mode: mode.into(),
        dx: 0.,
        dy: 0.,
        wheel,
        x: 0.,
        y: 0.,
        width: 512,
        height: 512,
        focused: true,
    }
}

#[test]
fn npr_headless_gallery_and_cube_do_not_spawn_companions_or_old_panels() {
    for scene in ["gallery", "cube"] {
        let runtime = runtime(scene);
        if scene == "cube" {
            runtime
                .required::<PlaygroundCompanionService>()
                .unwrap()
                .enable(PathBuf::from(env!("CARGO_BIN_EXE_amigo-app")));
        }
        tick(&runtime);
        assert!(
            runtime
                .required::<PlaygroundCompanionService>()
                .unwrap()
                .connections()
                .is_empty()
        );
        assert!(
            runtime
                .required::<amigo_panels::PanelService>()
                .unwrap()
                .connection_snapshot("npr")
                .is_none()
        );
    }
}

#[test]
fn npr_sidecars_hydrate_without_a_companion() {
    let gallery = runtime("gallery");
    tick(&gallery);
    let service = gallery.required::<NprPlaygroundService>().unwrap();
    assert_eq!(service.domain_snapshot().scene.as_deref(), Some("gallery"));
    assert!(service.domain_snapshot().settings.objects.is_empty());
    assert_eq!(service.domain_snapshot().settings.camera_distance, 14.);
    let cube = runtime("cube");
    tick(&cube);
    let state = cube.required::<NprPlaygroundState>().unwrap().snapshot();
    assert_eq!(state.objects.len(), 1);
    assert_eq!(state.selected, "cube");
    assert_eq!(state.camera_distance, 5.);
}

#[test]
fn npr_canvas_zoom_uses_host_frames_and_survives_pause_and_fit() {
    let runtime = runtime("gallery");
    tick(&runtime);
    let service = runtime.required::<NprPlaygroundService>().unwrap();
    let systems = runtime.required::<SystemRegistry>().unwrap();
    runtime
        .required::<amigo_session::RuntimeFrameClockService>()
        .unwrap()
        .force_single_simulation_tick(1. / 60.);
    edit(
        &service,
        NprPlaygroundIntent::SelectModel {
            model: "cube".into(),
        },
    );
    edit(
        &service,
        NprPlaygroundIntent::SetMotion {
            paused: true,
            speed: 1.,
            sketch_paused: false,
        },
    );
    let before = service.domain_snapshot().settings.camera_distance;
    edit(&service, navigate("zoom", 38.));
    let state = runtime.required::<NprPlaygroundState>().unwrap();
    let first = state.snapshot().camera_distance;
    let authored = service.domain_snapshot();
    for _ in 0..4 {
        systems.run_phase(SystemPhase::Update, &runtime).unwrap();
    }
    assert_eq!(state.snapshot().camera_distance, first);
    let revision_before_easing = service.domain_snapshot().revision;
    service.advance_camera(1.0 / 60.0);
    assert_eq!(service.domain_snapshot().revision, revision_before_easing);
    for _ in 0..90 {
        systems
            .run_phase(SystemPhase::PostUpdate, &runtime)
            .unwrap();
    }
    let settled = state.snapshot().camera_distance;
    assert_eq!(
        service.domain_snapshot().settings.camera_distance,
        authored.settings.camera_distance
    );
    assert!(settled < first && first < before);
    assert!((settled - before * (-0.12_f32).exp()).abs() < 0.0001);
    edit(&service, navigate("zoom", -76.));
    edit(&service, navigate("fit", 0.));
    let fitted = service.domain_snapshot().settings.camera_distance;
    for _ in 0..5 {
        systems
            .run_phase(SystemPhase::PostUpdate, &runtime)
            .unwrap();
    }
    assert_eq!(service.domain_snapshot().settings.camera_distance, fitted);
}

#[test]
fn npr_rhai_adapter_edits_the_same_session_and_supports_undo() {
    let runtime = runtime("gallery");
    tick(&runtime);
    let scripts = runtime
        .required::<amigo_scripting_api::ScriptRuntimeService>()
        .unwrap();
    let service = runtime.required::<NprPlaygroundService>().unwrap();
    let context = || amigo_scripting_api::DevConsoleScriptContext::new(Some("gallery".into()));
    scripts.eval_console(context(),r#"npr_playground_metadata(); npr_playground_dispatch(#{kind:"set_motion",paused:false,speed:2.0,sketch_paused:false});"#).unwrap();
    assert_eq!(service.domain_snapshot().settings.speed, 2.);
    scripts
        .eval_console(context(), r#"npr_playground_dispatch(#{kind:"undo"});"#)
        .unwrap();
    assert_eq!(service.domain_snapshot().settings.speed, 1.);
}

#[cfg(windows)]
#[test]
fn npr_native_companion_opens_once_and_scene_unload_closes_it() {
    let runtime = runtime("gallery");
    let companions = runtime.required::<PlaygroundCompanionService>().unwrap();
    companions.enable(PathBuf::from(env!("CARGO_BIN_EXE_amigo-app")));
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        tick(&runtime);
        let clients = companions.connections();
        assert!(clients.len() <= 1);
        if clients.first().is_some_and(|client| client.ready) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "companion failed: {:?}",
            companions.error()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(companions.close("npr-playground"));
    assert!(companions.owns_primary_window());
    assert!(companions.primary_window_closed());
    for _ in 0..5 {
        tick(&runtime);
    }
    assert!(
        companions.connections().is_empty(),
        "manual close must not reopen the client"
    );
    let host = runtime
        .required::<amigo_playground_api::PlaygroundHostService>()
        .unwrap();
    // Re-entering the scene may open a new client; unloading must close a live one.
    companions.load_scene(None, &[], host.clone()).unwrap();
    tick(&runtime);
    assert_eq!(companions.connections().len(), 1);
    companions
        .load_scene(Some("next-scene".into()), &[], host)
        .unwrap();
    assert!(companions.connections().is_empty());
    assert!(!companions.owns_primary_window());
    assert!(!companions.primary_window_closed());
}
