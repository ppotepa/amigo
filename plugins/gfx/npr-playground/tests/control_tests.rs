use amigo_npr_playground_plugin::{NprPlaygroundRenderService, NprPlaygroundState, state::PREFIX};
use amigo_panels::PresetProvider;
use amigo_runtime_control::{ControlValue, RuntimeControlService};
use std::sync::Arc;

#[test]
fn metadata_controls_validate_atomically_and_presets_restore_all_objects() {
    let state = Arc::new(NprPlaygroundState::default());
    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    let path = |field: &str| format!("{PREFIX}{field}");
    controls
        .set(&path("object.scale"), ControlValue::F64(2.0))
        .unwrap();
    assert!(
        controls
            .set(&path("object.scale"), ControlValue::F64(-1.0))
            .is_err()
    );
    assert!(controls.set(&path("fps"), ControlValue::F64(90.0)).is_err());
    assert!(
        controls
            .set(
                &path("global.ink"),
                ControlValue::Color([f32::NAN, 0.0, 0.0, 1.0])
            )
            .is_err()
    );
    assert_eq!(state.snapshot().objects["cube"].scale, 2.0);
    controls
        .set(&path("seed"), ControlValue::U64(u64::MAX))
        .unwrap();
    assert_eq!(
        controls.get(&path("seed")).unwrap(),
        ControlValue::U64(u64::MAX)
    );
    let saved = PresetProvider::snapshot(state.as_ref()).unwrap();
    let mut invalid = saved.clone();
    invalid["camera_distance"] = serde_yaml::to_value(-3.0).unwrap();
    assert!(state.apply(invalid).is_err());
    assert_eq!(PresetProvider::snapshot(state.as_ref()).unwrap(), saved);
    controls.reset(&path("object.scale")).unwrap();
    assert_eq!(state.snapshot().objects["cube"].scale, 1.0);
    state.apply(saved).unwrap();
    assert_eq!(state.snapshot().objects["cube"].scale, 2.0);
}

#[test]
fn pause_step_and_extract_do_not_advance_state() {
    let state = NprPlaygroundState::default();
    state.settings.lock().unwrap().paused = true;
    let before = state.snapshot().objects["cube"].rotation;
    state.tick(0.5);
    assert_eq!(state.snapshot().objects["cube"].rotation, before);
    state.settings.lock().unwrap().step = true;
    state.tick(0.5);
    let after = state.snapshot();
    assert_ne!(after.objects["cube"].rotation, before);
    assert!(!after.step);
    let render = NprPlaygroundRenderService::default();
    render.rebuild(&after, [512, 512]).unwrap();
    let first = render.snapshot().unwrap().packet;
    render.rebuild(&after, [512, 512]).unwrap();
    assert_eq!(first, render.snapshot().unwrap().packet);
    render.rebuild(&after, [320, 640]).unwrap();
    assert_eq!(render.snapshot().unwrap().packet.stats.viewport, [320, 640]);
    render.clear();
    assert!(render.commands().is_empty());
}

#[test]
fn gallery_imports_all_six_models_and_binds_the_authored_layout() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../mods/npr-playground");
    let state = Arc::new(NprPlaygroundState::default());
    state.configure_scene(true);
    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    let layout: amigo_panel_api::PanelDocument =
        serde_yaml::from_str(&std::fs::read_to_string(root.join("ui/npr.panel.yml")).unwrap())
            .unwrap();
    layout
        .validate_bindings(&controls.registry_snapshot())
        .unwrap();
    assert_eq!(layout.artwork.len(), 9);
    assert!(
        layout
            .artwork
            .values()
            .all(|triangles| !triangles.is_empty())
    );
    let mut transport = Vec::new();
    amigo_panel_api::write_message(&mut transport, &layout).unwrap();
    assert!(transport.len() < amigo_panel_api::MAX_FRAME_BYTES);
    let render = NprPlaygroundRenderService::default();
    render.load_models(&root).unwrap();
    render.rebuild(&state.snapshot(), [1024, 768]).unwrap();
    assert_eq!(render.commands().len(), 6);
    assert!(
        render
            .commands()
            .iter()
            .all(|c| !c.packet.fills.is_empty() && !c.packet.strokes.is_empty())
    );
    let annotated = render
        .commands()
        .iter()
        .map(|c| c.packet.fills.len())
        .sum::<usize>();
    let mut plain = state.snapshot();
    plain.highlight_selected = false;
    render.rebuild(&plain, [1024, 768]).unwrap();
    assert_eq!(
        annotated,
        render
            .commands()
            .iter()
            .map(|c| c.packet.fills.len())
            .sum::<usize>()
            + 2
    );
}

#[test]
fn workshop_history_rotation_scope_and_comparison_are_independent() {
    let state = Arc::new(NprPlaygroundState::default());
    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    let set = |key: &str, value| controls.set(&format!("{PREFIX}{key}"), value).unwrap();
    let action = |key| set(key, ControlValue::Bool(true));
    set("object.rotating", ControlValue::Bool(false));
    let before = state.snapshot();
    state.tick(0.5);
    assert_eq!(
        before.objects["cube"].rotation,
        state.snapshot().objects["cube"].rotation
    );
    assert_ne!(
        before.objects["sphere"].rotation,
        state.snapshot().objects["sphere"].rotation
    );
    set("object.scale", ControlValue::F64(2.0));
    state.tick(0.1);
    set("object.scale", ControlValue::F64(3.0));
    let live_rotation = state.snapshot().objects["sphere"].rotation;
    action("undo");
    assert_eq!(state.snapshot().objects["cube"].scale, 1.0);
    assert_eq!(state.snapshot().objects["sphere"].rotation, live_rotation);
    action("redo");
    assert_eq!(state.snapshot().objects["cube"].scale, 3.0);
    set("style_scope", ControlValue::String("Obiekt".into()));
    assert!(
        controls
            .set(
                &format!("{PREFIX}appearance.outline_width"),
                ControlValue::F64(9.0)
            )
            .is_err()
    );
    set("object.override_style", ControlValue::Bool(true));
    action("capture_before");
    set("appearance.outline_width", ControlValue::F64(9.0));
    assert_eq!(state.snapshot().objects["cube"].style.outline_width, 9.0);
    assert_eq!(state.snapshot().global.outline_width, 4.0);
    set("preview_before", ControlValue::Bool(true));
    assert_eq!(
        state.render_snapshot().objects["cube"].style.outline_width,
        4.0
    );
    assert!(
        controls
            .set(&format!("{PREFIX}object.scale"), ControlValue::F64(4.0))
            .is_err()
    );
    set("preview_before", ControlValue::Bool(false));
    assert_eq!(
        state.render_snapshot().objects["cube"].style.outline_width,
        9.0
    );
    action("reset_style");
    assert!(!state.snapshot().objects["cube"].override_style);
}

#[test]
fn typed_tool_profiles_are_exposed_and_validated() {
    let state = Arc::new(NprPlaygroundState::default());
    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    let path = |field: &str| format!("{PREFIX}{field}");
    controls
        .set(
            &path("style_preset"),
            ControlValue::String("Pencil Study".into()),
        )
        .unwrap();
    assert_eq!(
        controls.get(&path("global.tool")).unwrap(),
        ControlValue::String("pencil".into())
    );
    assert!(
        controls
            .set(&path("global.gesture_confidence"), ControlValue::F64(1.5))
            .is_err()
    );
    assert!(
        controls
            .set(
                &path("global.tool"),
                ControlValue::String("not-a-tool".into())
            )
            .is_err()
    );
    assert!(state.snapshot().global.paper_tooth > 0.5);
}

#[test]
fn selection_fits_only_single_mode_and_undo_targets_stable_object_ids() {
    let state = Arc::new(NprPlaygroundState::default());
    state.configure_scene(true);
    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    let set = |key: &str, value| controls.set(&format!("{PREFIX}{key}"), value).unwrap();
    set("object.scale", ControlValue::F64(2.0));
    let camera = state.snapshot().camera_target;
    set("selected", ControlValue::String("sphere".into()));
    assert_eq!(state.snapshot().camera_target, camera);
    set("object.scale", ControlValue::F64(3.0));
    set("undo", ControlValue::Bool(true));
    assert_eq!(state.snapshot().objects["sphere"].scale, 1.0);
    assert_eq!(state.snapshot().objects["cube"].scale, 2.0);
    set("focus_selected", ControlValue::Bool(true));
    assert!(!state.snapshot().gallery);
    assert_eq!(
        state.snapshot().camera_target,
        state.snapshot().objects["sphere"].position
    );
    set("selected", ControlValue::String("wedge".into()));
    assert_eq!(
        state.snapshot().camera_target,
        state.snapshot().objects["wedge"].position
    );
}

#[test]
fn look_presets_preserve_scene_and_are_atomic_and_undoable() {
    use amigo_npr_playground_plugin::state::look_presets::LookPresetProvider;
    let state = Arc::new(NprPlaygroundState::default());
    let looks = LookPresetProvider(state.clone());
    let mut saved = looks.snapshot().unwrap();
    saved["outline_width"] = serde_yaml::to_value(8.0).unwrap();
    let before = state.snapshot();
    looks.apply(saved.clone()).unwrap();
    let after = state.snapshot();
    assert_eq!(after.global.outline_width, 8.0);
    assert_eq!(after.global.paper, before.global.paper);
    assert_eq!(after.global.light_direction, before.global.light_direction);
    assert_eq!(after.objects, before.objects);
    assert_eq!(after.camera_target, before.camera_target);
    saved["outline_width"] = serde_yaml::to_value(-1.0).unwrap();
    assert!(looks.apply(saved).is_err());
    assert_eq!(state.snapshot(), after);
    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    controls
        .set(&format!("{PREFIX}undo"), ControlValue::Bool(true))
        .unwrap();
    assert_eq!(state.snapshot(), before);
}

#[test]
fn render_diagnostics_report_the_effective_typed_style_preset() {
    use amigo_npr_playground_plugin::state::{Settings, style_preset_id};

    let settings = Settings::for_scene(false);
    assert_eq!(style_preset_id(settings.global), "comic-ink");

    let pencil = amigo_npr_playground_plugin::state::style_preset("Pencil Study")
        .expect("built-in pencil look");
    assert_eq!(style_preset_id(pencil), "pencil-study");

    let mut custom = pencil;
    custom.wobble += 0.01;
    assert_eq!(style_preset_id(custom), "custom");
}
