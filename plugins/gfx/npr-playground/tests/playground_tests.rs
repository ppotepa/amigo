use amigo_npr_playground_plugin::{NprPlaygroundState, playground::*, state::ObjectSettings};
use amigo_playground_api::*;
use std::sync::Arc;

#[test]
fn active_playground_snapshot_publishes_renderer_generated_preview_channels() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../mods/npr-playground");
    let service = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    service
        .open_scene(&root, std::path::Path::new("scenes/gallery/npr.scene.yml"))
        .unwrap();
    let mut received = false;
    for _ in 0..40 {
        let snapshot = PlaygroundProvider::snapshot(&service);
        let looks = snapshot
            .values
            .get("look_previews")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|items| items.values().any(serde_json::Value::is_string));
        let brushes = snapshot
            .values
            .get("brush_previews")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|items| items.values().any(serde_json::Value::is_string));
        if looks && brushes {
            received = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        received,
        "preview queues must publish images to the companion snapshot"
    );
}

#[test]
fn camera_drag_records_one_undo_and_conflict_ends_grouping() {
    let service = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    let before = service.domain_snapshot().settings.camera_yaw;
    let send = |intent| {
        service
            .dispatch_intent(1, service.revision(), "viewport".into(), intent)
            .unwrap()
    };
    send(NprPlaygroundIntent::BeginCameraGesture { gesture_id: 1 });
    let movement = || NprPlaygroundIntent::Navigate {
        mode: "orbit".into(),
        dx: 10.,
        dy: 0.,
        wheel: 0.,
        x: 0.,
        y: 0.,
        width: 640,
        height: 360,
        focused: true,
    };
    for _ in 0..5 {
        send(movement());
    }
    send(NprPlaygroundIntent::EndCameraGesture { gesture_id: 1 });
    assert!((service.domain_snapshot().settings.camera_yaw - before - 15.).abs() < 0.001);
    send(NprPlaygroundIntent::Undo);
    assert_eq!(service.domain_snapshot().settings.camera_yaw, before);
    assert!(!service.domain_snapshot().can_undo);
    send(NprPlaygroundIntent::BeginCameraGesture { gesture_id: 2 });
    send(movement());
    assert!(
        service
            .dispatch_intent(99, 0, "viewport".into(), movement())
            .is_err()
    );
    send(movement());
    send(NprPlaygroundIntent::Undo);
    assert!((service.domain_snapshot().settings.camera_yaw - before - 3.).abs() < 0.001);
}

#[test]
fn typed_intents_validate_before_mutation_and_undo_redo_are_versioned() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let before = state.snapshot();
    let error = service
        .dispatch_intent(
            1,
            0,
            "speed".into(),
            NprPlaygroundIntent::SetMotion {
                paused: true,
                speed: 99.0,
                sketch_paused: false,
            },
        )
        .unwrap_err();
    assert_eq!(error.control, "speed");
    assert_eq!(state.snapshot(), before);
    assert_eq!(service.domain_snapshot().revision, 0);
    service
        .dispatch_intent(
            2,
            0,
            "pause".into(),
            NprPlaygroundIntent::SetMotion {
                paused: true,
                speed: 1.0,
                sketch_paused: false,
            },
        )
        .unwrap();
    assert!(state.snapshot().paused);
    assert!(service.domain_snapshot().dirty);
    service
        .dispatch_intent(3, 1, "undo".into(), NprPlaygroundIntent::Undo)
        .unwrap();
    assert_eq!(state.snapshot(), before);
    service
        .dispatch_intent(4, 2, "redo".into(), NprPlaygroundIntent::Redo)
        .unwrap();
    assert!(state.snapshot().paused);
    assert!(
        service
            .dispatch_intent(5, 1, "pause".into(), NprPlaygroundIntent::Undo)
            .is_err()
    );
}

#[test]
fn build_up_is_editor_state_and_variants_are_undoable() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "build-up".into(),
            NprPlaygroundIntent::SetBuildUp { value: 0.25 },
        )
        .unwrap();
    assert_eq!(*state.build_up.lock().unwrap(), 0.25);
    assert!(!service.domain_snapshot().dirty);
    service
        .dispatch_intent(
            2,
            service.revision(),
            "variant".into(),
            NprPlaygroundIntent::SaveVariant {
                id: "before".into(),
            },
        )
        .unwrap();
    assert!(service.domain_snapshot().variants.contains_key("before"));
    let mut changed = state.snapshot();
    changed.style_layers.layer_mut("contours").unwrap().opacity = 0.42;
    service
        .dispatch_intent(
            3,
            service.revision(),
            "look".into(),
            NprPlaygroundIntent::SetLook {
                style: changed.global,
                layers: changed.style_layers,
            },
        )
        .unwrap();
    // Applying a variant is a single history operation and restores the saved look.
    let revision = service.revision();
    service
        .dispatch_intent(
            4,
            revision,
            "variant".into(),
            NprPlaygroundIntent::ApplyVariant {
                id: "before".into(),
            },
        )
        .unwrap();
    assert!(service.domain_snapshot().can_undo);
}

#[test]
fn brush_version_save_is_explicit_and_becomes_a_document_change() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state);
    let brush = amigo_render_npr::BrushDefinition {
        id: "ink-liner".into(),
        version: 2,
        name: "Ink liner custom".into(),
        ..Default::default()
    };
    service
        .dispatch_intent(
            1,
            0,
            "brush".into(),
            NprPlaygroundIntent::SaveBrushVersion { brush },
        )
        .unwrap();
    assert!(service.domain_snapshot().dirty);
    assert!(
        service
            .domain_snapshot()
            .brushes
            .resolve(&amigo_render_npr::BrushReference {
                id: "ink-liner".into(),
                version: 2
            })
            .is_ok()
    );
    let bad = amigo_render_npr::BrushDefinition {
        id: "ink-liner".into(),
        version: 4,
        ..Default::default()
    };
    assert!(
        service
            .dispatch_intent(
                2,
                service.revision(),
                "brush".into(),
                NprPlaygroundIntent::SaveBrushVersion { brush: bad }
            )
            .is_err()
    );
}

#[test]
fn brush_version_update_uses_document_library_and_is_one_undo_operation() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "brush".into(),
            NprPlaygroundIntent::SaveBrushVersion {
                brush: amigo_render_npr::BrushDefinition {
                    id: "ink-liner".into(),
                    version: 2,
                    name: "Local ink v2".into(),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let before = state.snapshot();
    service
        .dispatch_intent(
            2,
            service.revision(),
            "brush".into(),
            NprPlaygroundIntent::UpdateBrushVersion {
                id: "ink-liner".into(),
                from: 1,
                to: 2,
            },
        )
        .unwrap();
    assert!(
        state
            .snapshot()
            .style_layers
            .layers
            .iter()
            .filter_map(|layer| layer.brush.as_ref())
            .any(|brush| brush.brush.id == "ink-liner" && brush.brush.version == 2)
    );
    service
        .dispatch_intent(
            3,
            service.revision(),
            "undo".into(),
            NprPlaygroundIntent::Undo,
        )
        .unwrap();
    assert_eq!(state.snapshot(), before);
}

#[test]
fn custom_brush_version_survives_draft_checkpoint_and_restore() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "brush".into(),
            NprPlaygroundIntent::SaveBrushVersion {
                brush: amigo_render_npr::BrushDefinition {
                    id: "ink-liner".into(),
                    version: 2,
                    name: "Draft-safe ink".into(),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    service
        .dispatch_intent(
            2,
            service.revision(),
            "brush".into(),
            NprPlaygroundIntent::UpdateBrushVersion {
                id: "ink-liner".into(),
                from: 1,
                to: 2,
            },
        )
        .unwrap();
    service
        .dispatch_intent(
            3,
            service.revision(),
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "sphere".into(),
            },
        )
        .unwrap();
    let draft = &service.domain_snapshot().drafts["cube"].settings;
    assert!(draft
        .brushes
        .resolve(&amigo_render_npr::BrushReference {
            id: "ink-liner".into(),
            version: 2,
        })
        .is_ok());
    service
        .dispatch_intent(
            4,
            service.revision(),
            "draft".into(),
            NprPlaygroundIntent::OpenDraft {
                model: "cube".into(),
            },
        )
        .unwrap();
    assert!(state
        .snapshot()
        .brushes
        .resolve(&amigo_render_npr::BrushReference {
            id: "ink-liner".into(),
            version: 2,
        })
        .is_ok());
}

#[test]
fn custom_brush_version_survives_save_and_reload() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("npr.scene.yml");
    std::fs::write(&path, "version: 1\nactive_look: null\nlook: {}\n").unwrap();
    let service = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    service
        .open_scene(root.path(), std::path::Path::new("npr.scene.yml"))
        .unwrap();
    service
        .dispatch_intent(
            1,
            service.revision(),
            "brush".into(),
            NprPlaygroundIntent::SaveBrushVersion {
                brush: amigo_render_npr::BrushDefinition {
                    id: "ink-liner".into(),
                    version: 2,
                    name: "Persistent ink".into(),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    service
        .dispatch_intent(
            2,
            service.revision(),
            "save".into(),
            NprPlaygroundIntent::SaveAll,
        )
        .unwrap();
    service
        .dispatch_intent(
            3,
            service.revision(),
            "reload".into(),
            NprPlaygroundIntent::Reload,
        )
        .unwrap();
    assert!(service
        .domain_snapshot()
        .brushes
        .resolve(&amigo_render_npr::BrushReference {
            id: "ink-liner".into(),
            version: 2,
        })
        .is_ok());
}

#[test]
fn flat_layer_stack_round_trip_covers_duplicate_mask_and_undo_redo() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let before = state.snapshot();
    let mut layers = before.style_layers.clone();
    let source = layers
        .layers
        .iter()
        .find(|layer| layer.id == "contours")
        .cloned()
        .unwrap();
    let mut duplicate = source.clone();
    duplicate.id = "contours-copy".into();
    duplicate.label = "Contours copy".into();
    duplicate.mask = amigo_render_npr::CoverageMask::Noise {
        amount: 0.25,
        seed: 77,
        invert: false,
    };
    let index = layers
        .layers
        .iter()
        .position(|layer| layer.id == "contours")
        .unwrap();
    layers.layers.insert(index + 1, duplicate);
    service
        .dispatch_intent(
            1,
            0,
            "layers".into(),
            NprPlaygroundIntent::SetLook {
                style: before.global,
                layers: layers.clone(),
            },
        )
        .unwrap();
    let changed = state.snapshot();
    let copy = changed
        .style_layers
        .layers
        .iter()
        .find(|layer| layer.id == "contours-copy")
        .unwrap();
    assert!(matches!(
        copy.mask,
        amigo_render_npr::CoverageMask::Noise { seed: 77, .. }
    ));

    service
        .dispatch_intent(
            2,
            service.revision(),
            "undo".into(),
            NprPlaygroundIntent::Undo,
        )
        .unwrap();
    assert_eq!(state.snapshot().style_layers, before.style_layers);
    service
        .dispatch_intent(
            3,
            service.revision(),
            "redo".into(),
            NprPlaygroundIntent::Redo,
        )
        .unwrap();
    assert_eq!(state.snapshot().style_layers, layers);

    let encoded = serde_json::to_string(&state.snapshot().style_layers).unwrap();
    let decoded: amigo_render_npr::NprStyleLayers = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, layers);
}

#[test]
fn solo_layer_is_editor_state_and_keeps_paper_visible() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "layers".into(),
            NprPlaygroundIntent::SetSoloLayer {
                layer: Some("contours".into()),
            },
        )
        .unwrap();
    let rendered = state.render_snapshot();
    assert!(rendered.style_layers.layer("paper").unwrap().enabled);
    assert!(rendered.style_layers.layer("contours").unwrap().enabled);
    assert!(!rendered.style_layers.layer("hatching").unwrap().enabled);
    assert!(!service.domain_snapshot().dirty);
    service
        .dispatch_intent(
            2,
            service.revision(),
            "layers".into(),
            NprPlaygroundIntent::SetSoloLayer { layer: None },
        )
        .unwrap();
    assert!(
        state
            .render_snapshot()
            .style_layers
            .layer("hatching")
            .unwrap()
            .enabled
    );
}

#[test]
fn canvas_input_requires_focus_and_selection_events_use_shared_service() {
    let service = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    assert!(
        service
            .dispatch_intent(
                1,
                0,
                "viewport".into(),
                NprPlaygroundIntent::Navigate {
                    mode: "orbit".into(),
                    dx: 20.,
                    dy: 10.,
                    wheel: 0.,
                    x: 0.,
                    y: 0.,
                    width: 800,
                    height: 600,
                    focused: false
                }
            )
            .is_err()
    );
    service
        .dispatch_intent(
            2,
            0,
            "selection".into(),
            NprPlaygroundIntent::SelectModel {
                model: "sphere".into(),
            },
        )
        .unwrap();
    assert!(
        service
            .drain_events()
            .iter()
            .any(|e| matches!(e,PlaygroundEvent::Domain{name,..} if name=="selection_changed"))
    );
}

#[test]
fn camera_navigation_updates_orbit_pan_zoom_and_selected_spin() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let before = state.snapshot();

    let navigate =
        |request_id: u64, base_revision: u64, mode: &str, dx: f32, dy: f32, wheel: f32| {
            service
                .dispatch_intent(
                    request_id,
                    base_revision,
                    "viewport".into(),
                    NprPlaygroundIntent::Navigate {
                        mode: mode.into(),
                        dx,
                        dy,
                        wheel,
                        x: 0.,
                        y: 0.,
                        width: 800,
                        height: 600,
                        focused: true,
                    },
                )
                .unwrap();
        };

    navigate(1, 0, "orbit", 20., 10., 0.);
    let orbit = state.snapshot();
    assert_ne!(orbit.camera_yaw, before.camera_yaw);
    assert_ne!(orbit.camera_pitch, before.camera_pitch);

    navigate(2, 1, "pan", 12., -8., 0.);
    let pan = state.snapshot();
    assert_ne!(pan.camera_target, orbit.camera_target);

    navigate(3, 2, "zoom", 0., 0., 24.);
    let zoom = state.snapshot();
    assert!(zoom.camera_distance < pan.camera_distance);

    let object = zoom.objects.get(&zoom.selected).unwrap().clone();
    service
        .dispatch_intent(
            4,
            3,
            "object.rotating".into(),
            NprPlaygroundIntent::SetObject {
                object: zoom.selected.clone(),
                settings: ObjectSettings {
                    rotating: !object.rotating,
                    ..object
                },
            },
        )
        .unwrap();
    assert_ne!(
        state.snapshot().objects[&state.snapshot().selected].rotating,
        before.objects[&before.selected].rotating
    );
}

#[test]
fn saving_sets_history_baseline_and_conflicts_preserve_edits() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("scenes/gallery")).unwrap();
    std::fs::write(
        root.path().join("scenes/gallery/npr.scene.yml"),
        "version: 1\nactive_look: null\nlook: {}\n",
    )
    .unwrap();
    let service = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    service
        .open_scene(
            root.path(),
            std::path::Path::new("scenes/gallery/npr.scene.yml"),
        )
        .unwrap();
    service
        .dispatch_intent(
            1,
            1,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "cube".into(),
            },
        )
        .unwrap();
    service
        .dispatch_intent(2, 2, "save".into(), NprPlaygroundIntent::SaveAll)
        .unwrap();
    assert!(!service.domain_snapshot().dirty);
    assert!(!service.domain_snapshot().can_undo);
    service
        .dispatch_intent(
            3,
            3,
            "pause".into(),
            NprPlaygroundIntent::SetMotion {
                paused: true,
                speed: 1.,
                sketch_paused: false,
            },
        )
        .unwrap();
    std::fs::write(
        root.path().join("scenes/gallery/npr.scene.yml"),
        "external edit",
    )
    .unwrap();
    assert!(
        service
            .dispatch_intent(4, 4, "save".into(), NprPlaygroundIntent::SaveAll)
            .is_err()
    );
    assert!(service.domain_snapshot().dirty);
    assert!(service.domain_snapshot().can_undo);
}

#[test]
fn scene_model_browser_replaces_the_single_viewer_object() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "sphere".into(),
            },
        )
        .unwrap();
    assert_eq!(state.snapshot().objects.len(), 1);
    assert_eq!(state.snapshot().selected, "sphere");
    service
        .dispatch_intent(
            2,
            1,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "cube".into(),
            },
        )
        .unwrap();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.objects.len(), 1);
    assert_eq!(snapshot.selected, "cube");
    assert_eq!(snapshot.objects["cube"].model, "cube");
}

#[test]
fn source_selection_rejects_unknown_models_without_an_asset_catalog() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let before = state.snapshot();

    assert!(service
        .dispatch_intent(
            1,
            0,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "not-a-source".into(),
            },
        )
        .unwrap_err()
        .message
        .contains("unknown built-in source model"));
    assert_eq!(state.snapshot(), before);
}

#[test]
fn object_updates_cannot_bypass_basic_source_selection() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let before = state.snapshot();
    let mut settings = before.objects["cube"].clone();
    settings.model = "not-a-source".into();

    assert!(service
        .dispatch_intent(
            1,
            0,
            "object.model".into(),
            NprPlaygroundIntent::SetObject {
                object: "cube".into(),
                settings,
            },
        )
        .unwrap_err()
        .message
        .contains("SelectModel"));
    assert_eq!(state.snapshot(), before);
}

#[test]
fn selecting_another_model_checkpoints_dirty_drawing_and_starts_clean_layers() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "sphere".into(),
            },
        )
        .unwrap();
    let mut edited = state.snapshot();
    edited.style_layers.layer_mut("contours").unwrap().opacity = 0.23;
    service
        .dispatch_intent(
            2,
            1,
            "layers".into(),
            NprPlaygroundIntent::SetLook {
                style: edited.global,
                layers: edited.style_layers,
            },
        )
        .unwrap();
    service
        .dispatch_intent(
            3,
            2,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "cube".into(),
            },
        )
        .unwrap();

    let snapshot = service.domain_snapshot();
    assert_eq!(snapshot.drafts["sphere"].source_model, "sphere");
    assert_eq!(
        snapshot.drafts["sphere"]
            .settings
            .style_layers
            .layer("contours")
            .unwrap()
            .opacity,
        0.23
    );
    assert_eq!(snapshot.settings.selected, "cube");
    assert_eq!(
        snapshot.settings.style_layers,
        amigo_render_npr::NprStyleLayers::default()
    );
    assert!(!snapshot.dirty);
}

#[test]
fn opening_a_draft_restores_its_model_bound_layer_stack() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "sphere".into(),
            },
        )
        .unwrap();
    let mut edited = state.snapshot();
    edited.style_layers.layer_mut("contours").unwrap().opacity = 0.41;
    service
        .dispatch_intent(
            2,
            1,
            "layers".into(),
            NprPlaygroundIntent::SetLook {
                style: edited.global,
                layers: edited.style_layers,
            },
        )
        .unwrap();
    service
        .dispatch_intent(
            3,
            2,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "cube".into(),
            },
        )
        .unwrap();
    service
        .dispatch_intent(
            4,
            3,
            "draft".into(),
            NprPlaygroundIntent::OpenDraft {
                model: "sphere".into(),
            },
        )
        .unwrap();

    let snapshot = service.domain_snapshot();
    assert_eq!(snapshot.settings.selected, "sphere");
    assert_eq!(
        snapshot
            .settings
            .style_layers
            .layer("contours")
            .unwrap()
            .opacity,
        0.41
    );
    assert!(!snapshot.dirty);
}

#[test]
fn model_browser_never_accumulates_objects_from_a_gallery_profile() {
    let state = Arc::new(NprPlaygroundState::default());
    *state.settings.lock().unwrap() =
        amigo_npr_playground_plugin::state::Settings::empty_scene(true);
    let service = NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "sphere".into(),
            },
        )
        .unwrap();
    service
        .dispatch_intent(
            2,
            1,
            "models".into(),
            NprPlaygroundIntent::SelectModel {
                model: "cube".into(),
            },
        )
        .unwrap();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.objects.len(), 1);
    assert_eq!(snapshot.selected, "cube");
}

#[test]
fn authored_looks_keep_includes_and_look_selection_is_undoable() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("npr/looks")).unwrap();
    std::fs::write(
        root.path().join("npr.scene.yml"),
        "version: 1\nactive_look: base\nlook: {}\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("npr/looks/base.npr-look.yml"),
        "id: base\nlook: {}\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("npr/looks/pencil.npr-look.yml"),
        "id: pencil\nincludes: [base]\nlook:\n  style:\n    tool: pencil\n",
    )
    .unwrap();
    let service = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    service
        .open_scene(root.path(), std::path::Path::new("npr.scene.yml"))
        .unwrap();
    let dispatch = |intent| {
        service
            .dispatch_intent(1, service.domain_snapshot().revision, "look".into(), intent)
            .unwrap()
    };
    dispatch(NprPlaygroundIntent::UseLook {
        id: "pencil".into(),
    });
    assert_eq!(
        service.domain_snapshot().active_look.as_deref(),
        Some("pencil")
    );
    dispatch(NprPlaygroundIntent::Undo);
    assert_eq!(
        service.domain_snapshot().active_look.as_deref(),
        Some("base")
    );
    dispatch(NprPlaygroundIntent::Redo);
    let mut settings = service.domain_snapshot().settings;
    settings.global.outline_width = 5.;
    dispatch(NprPlaygroundIntent::SetLook {
        style: settings.global,
        layers: settings.style_layers,
    });
    dispatch(NprPlaygroundIntent::SaveLook);
    let document: amigo_npr_playground_plugin::documents::NprLookDocument = serde_yaml::from_slice(
        &std::fs::read(root.path().join("npr/looks/pencil.npr-look.yml")).unwrap(),
    )
    .unwrap();
    assert_eq!(document.includes, vec!["base"]);
    assert!(!document.look.style.contains_key("paper"));
    dispatch(NprPlaygroundIntent::SaveAll);
    dispatch(NprPlaygroundIntent::Reload);
    assert_eq!(service.domain_snapshot().settings.global.outline_width, 5.);
}

#[test]
fn locked_layers_reject_parameter_changes_from_any_adapter() {
    let service = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    service
        .dispatch_intent(
            1,
            0,
            "layer.lock".into(),
            NprPlaygroundIntent::SetLayerLock {
                layer: "fill".into(),
                locked: true,
            },
        )
        .unwrap();
    let mut settings = service.domain_snapshot().settings;
    settings.style_layers.layer_mut("fill").unwrap().opacity = 0.3;
    assert!(
        service
            .dispatch_intent(
                2,
                1,
                "layer.opacity".into(),
                NprPlaygroundIntent::SetLook {
                    style: settings.global,
                    layers: settings.style_layers
                }
            )
            .unwrap_err()
            .message
            .contains("locked")
    );
    assert_eq!(service.domain_snapshot().revision, 1);
}
