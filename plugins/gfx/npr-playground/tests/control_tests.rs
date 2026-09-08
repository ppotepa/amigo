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
        .set(
            &path("motion.appearance_fade_seconds"),
            ControlValue::F64(0.0),
        )
        .unwrap();
    assert_eq!(state.snapshot().motion.appearance_fade_seconds, 0.0);
    assert!(
        controls
            .set(
                &path("motion.appearance_fade_seconds"),
                ControlValue::F64(2.1)
            )
            .is_err()
    );
    controls
        .set(
            &path("motion.mode"),
            ControlValue::String("redraw-on-motion".into()),
        )
        .unwrap();
    assert_eq!(
        controls.get(&path("motion_redraw_editable")).unwrap(),
        ControlValue::Bool(true)
    );
    controls
        .set(&path("motion.redraw_hz"), ControlValue::F64(6.0))
        .unwrap();
    assert_eq!(
        controls.get(&path("motion.mode")).unwrap(),
        ControlValue::String("redraw-on-motion".into())
    );
    assert!(
        controls
            .set(&path("motion.redraw_strength"), ControlValue::F64(1.1))
            .is_err()
    );
    controls
        .set(
            &path("global.min_crease_length_pixels"),
            ControlValue::F64(64.0),
        )
        .unwrap();
    assert_eq!(
        controls
            .get(&path("global.min_crease_length_pixels"))
            .unwrap(),
        ControlValue::F64(64.0)
    );
    assert!(controls
        .set(
            &path("global.min_crease_length_pixels"),
            ControlValue::F64(64.5)
        )
        .is_err());
    controls
        .set(
            &path("global.min_form_line_confidence"),
            ControlValue::F64(0.55),
        )
        .unwrap();
    assert!(controls
        .set(
            &path("global.min_form_line_confidence"),
            ControlValue::F64(1.01)
        )
        .is_err());
    controls
        .set(
            &path("global.suggestive_contours"),
            ControlValue::Bool(true),
        )
        .unwrap();
    assert_eq!(
        controls.get(&path("global.suggestive_contours")).unwrap(),
        ControlValue::Bool(true)
    );
    assert!(controls
        .set(
            &path("global.suggestive_contour_confidence"),
            ControlValue::F64(-0.01)
        )
        .is_err());
    controls
        .set(
            &path("object.surface_mode"),
            ControlValue::String("smooth".into()),
        )
        .unwrap();
    assert_eq!(
        controls.get(&path("object.surface_mode")).unwrap(),
        ControlValue::String("smooth".into())
    );
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
fn interactive_extract_eases_only_new_stroke_identities() {
    let state = NprPlaygroundState::default();
    let render = NprPlaygroundRenderService::default();
    let single = state.snapshot();
    render
        .rebuild_with_delta(&single, [512, 512], 1.0 / 60.0)
        .unwrap();
    let mut gallery = single.clone();
    gallery.gallery = true;
    for (id, object) in &mut gallery.objects {
        object.visible = id == "cube" || id == "wedge";
    }
    render
        .rebuild_with_delta(&gallery, [512, 512], 0.06)
        .unwrap();
    let commands = render.commands();
    assert!(commands.len() > 1);
    assert!(commands[0].packet.stats.temporal_retained_strokes > 0);
    assert!(
        commands[1].packet.stats.temporal_entering_strokes > 0,
        "new={:?}, strokes={}",
        commands[1].packet.stats.temporal_entering_strokes,
        commands[1].packet.strokes.len(),
    );
}

#[test]
fn stroke_motion_mode_changes_variants_only_when_explicitly_enabled() {
    use amigo_render_npr::StrokeMotionMode;

    let render = NprPlaygroundRenderService::default();
    let mut settings = NprPlaygroundState::default().snapshot();
    settings.motion.mode = StrokeMotionMode::RedrawOnMotion;
    settings.motion.redraw_hz = 4.0;
    render
        .rebuild_with_delta(&settings, [512, 512], 1.0 / 60.0)
        .unwrap();
    settings.objects.get_mut("cube").unwrap().rotation.y += 25.0;
    render
        .rebuild_with_delta(&settings, [512, 512], 1.0 / 60.0)
        .unwrap();
    assert!(render.commands()[0].packet.stats.gesture_variant_epoch > 0);

    settings.motion.mode = StrokeMotionMode::Stable;
    settings.objects.get_mut("cube").unwrap().rotation.y += 25.0;
    render
        .rebuild_with_delta(&settings, [512, 512], 1.0 / 60.0)
        .unwrap();
    assert_eq!(render.commands()[0].packet.stats.gesture_variant_epoch, 0);
}

#[test]
fn material_edits_do_not_reset_stroke_identity_scope() {
    let render = NprPlaygroundRenderService::default();
    let mut settings = NprPlaygroundState::default().snapshot();
    render
        .rebuild_with_delta(&settings, [512, 512], 1.0 / 60.0)
        .unwrap();
    settings.global.ink.x = 0.25;
    settings.global.paper.y = 0.75;
    render
        .rebuild_with_delta(&settings, [512, 512], 1.0 / 60.0)
        .unwrap();
    assert!(render.commands()[0].packet.stats.temporal_retained_strokes > 0);
    assert_eq!(
        render.commands()[0].packet.stats.temporal_entering_strokes,
        0
    );
}

#[test]
fn manual_gesture_variant_is_explicit_and_undoable() {
    let state = Arc::new(NprPlaygroundState::default());
    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    let path = |field: &str| format!("{PREFIX}{field}");
    controls
        .set(&path("new_gesture_variant"), ControlValue::Bool(true))
        .unwrap();
    assert_eq!(state.snapshot().objects["cube"].gesture_variant, 1);
    controls
        .set(&path("undo"), ControlValue::Bool(true))
        .unwrap();
    assert_eq!(state.snapshot().objects["cube"].gesture_variant, 0);
}

#[test]
fn manual_gesture_variant_changes_gesture_not_surface_identity() {
    use amigo_render_npr::StrokeRole;
    use std::collections::BTreeSet;

    let state = Arc::new(NprPlaygroundState::default());
    state.settings.lock().unwrap().global =
        amigo_npr_playground_plugin::state::style_preset("Pencil Study").unwrap();
    let render = NprPlaygroundRenderService::default();
    render.rebuild(&state.snapshot(), [512, 512]).unwrap();
    let before = render.commands()[0].packet.clone();
    let before_ids = before
        .strokes
        .iter()
        .filter(|stroke| stroke.role == StrokeRole::Tone)
        .map(|stroke| stroke.id)
        .collect::<BTreeSet<_>>();

    let controls = RuntimeControlService::default();
    controls.register_provider(state.clone());
    controls
        .set(
            &format!("{PREFIX}new_gesture_variant"),
            ControlValue::Bool(true),
        )
        .unwrap();
    render.rebuild(&state.snapshot(), [512, 512]).unwrap();
    let after = render.commands()[0].packet.clone();
    let after_ids = after
        .strokes
        .iter()
        .filter(|stroke| stroke.role == StrokeRole::Tone)
        .map(|stroke| stroke.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(before_ids, after_ids);
    assert_ne!(before, after);
}

#[test]
fn surface_hatch_identities_survive_a_rigid_object_rotation() {
    use amigo_render_npr::StrokeRole;
    use glam::{EulerRot, Quat, Vec3};
    use std::collections::BTreeSet;

    let render = NprPlaygroundRenderService::default();
    let mut settings = NprPlaygroundState::default().snapshot();
    settings.global = amigo_npr_playground_plugin::state::style_preset("Pencil Study")
        .expect("typed pencil profile");
    let local_light = Vec3::new(-0.4, 0.7, 1.0).normalize();
    let object = settings.objects.get_mut("cube").unwrap();
    let rotation = object.rotation.map(f32::to_radians);
    settings.global.light_direction =
        Quat::from_euler(EulerRot::YXZ, rotation.y, rotation.x, rotation.z) * local_light;
    render.rebuild(&settings, [512, 512]).unwrap();
    let before = render.commands()[0]
        .packet
        .strokes
        .iter()
        .filter(|stroke| stroke.role == StrokeRole::Tone)
        .map(|stroke| stroke.id)
        .collect::<BTreeSet<_>>();
    assert!(!before.is_empty());

    let object = settings.objects.get_mut("cube").unwrap();
    object.rotation.y += 23.0;
    let rotation = object.rotation.map(f32::to_radians);
    settings.global.light_direction =
        Quat::from_euler(EulerRot::YXZ, rotation.y, rotation.x, rotation.z) * local_light;
    render.rebuild(&settings, [512, 512]).unwrap();
    let after = render.commands()[0]
        .packet
        .strokes
        .iter()
        .filter(|stroke| stroke.role == StrokeRole::Tone)
        .map(|stroke| stroke.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(after, before);
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
