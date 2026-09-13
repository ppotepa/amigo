use amigo_npr_playground_plugin::{
    NprPlaygroundRenderService, NprPlaygroundState, documents::*, playground::*, state::Settings,
};
use amigo_render_npr::{ComicInkOverrides, NprBlendMode, NprStyleLayerOverrides, StrokeTool};
use std::{collections::BTreeMap, sync::Arc};

fn edit(
    service: &NprPlaygroundService,
    intent: NprPlaygroundIntent,
) -> Result<u64, NprPlaygroundActionError> {
    service.dispatch_intent(
        1,
        service.domain_snapshot().revision,
        "test.control".into(),
        intent,
    )
}

#[test]
fn viewport_animation_tracks_visible_models_and_independent_sketch_clock() {
    let mut settings = Settings::for_scene();
    settings.objects.get_mut("cube").unwrap().rotating = true;
    settings.paused = true;
    settings.sketch_paused = true;
    assert!(!settings.needs_temporal_frames());
    settings.motion.mode = amigo_render_npr::StrokeMotionMode::RedrawContinuously;
    settings.sketch_paused = false;
    assert!(settings.needs_temporal_frames());
    settings.sketch_paused = true;
    settings.paused = false;
    assert!(settings.needs_temporal_frames());
    settings.speed = 0.;
    assert!(!settings.needs_temporal_frames());
    settings.sketch_paused = false;
    settings
        .objects
        .get_mut(&settings.selected.clone())
        .unwrap()
        .visible = false;
    assert!(!settings.needs_temporal_frames());
    settings.objects.clear();
    assert!(!settings.needs_temporal_frames());
}

#[test]
fn model_rotation_is_static_by_default_and_explicitly_opt_in() {
    let state = NprPlaygroundState::default();
    let before = state.snapshot().objects["cube"].rotation;
    state.tick(0.25);
    assert_eq!(state.snapshot().objects["cube"].rotation, before);
    state.settings.lock().unwrap().objects.get_mut("cube").unwrap().rotating = true;
    state.tick(0.25);
    assert_ne!(state.snapshot().objects["cube"].rotation, before);
}

#[test]
fn typed_object_edits_validate_atomically_and_undo_preserves_other_objects() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let original = state.snapshot();
    for scale in [0., -1., 11., f32::NAN] {
        let mut object = original.objects["cube"].clone();
        object.scale = scale;
        assert!(
            edit(
                &service,
                NprPlaygroundIntent::SetObject {
                    object: "cube".into(),
                    settings: object
                }
            )
            .is_err()
        );
        assert_eq!(state.snapshot(), original);
    }
    let mut object = original.objects["cube"].clone();
    object.scale = 2.;
    object.gesture_variant += 1;
    edit(
        &service,
        NprPlaygroundIntent::SetObject {
            object: "cube".into(),
            settings: object,
        },
    )
    .unwrap();
    assert_eq!(state.snapshot().objects.len(), 1);
    assert_eq!(state.snapshot().selected, "cube");
    edit(&service, NprPlaygroundIntent::Undo).unwrap();
    assert_eq!(state.snapshot(), original);
    edit(&service, NprPlaygroundIntent::Redo).unwrap();
    assert_eq!(state.snapshot().objects["cube"].scale, 2.);
}

#[test]
fn typed_tools_and_sparse_object_overrides_reach_render_contracts() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    for tool in [
        StrokeTool::Pencil,
        StrokeTool::Fineliner,
        StrokeTool::Nib,
        StrokeTool::Brush,
    ] {
        let settings = state.snapshot();
        let mut style = settings.global;
        style.tool = tool;
        edit(
            &service,
            NprPlaygroundIntent::SetLook {
                style,
                layers: settings.style_layers,
            },
        )
        .unwrap();
        assert_eq!(state.snapshot().global.tool, tool);
    }
    let mut object = state.snapshot().objects["cube"].clone();
    object.style_overrides.outline_width = Some(7.);
    edit(
        &service,
        NprPlaygroundIntent::SetObject {
            object: "cube".into(),
            settings: object,
        },
    )
    .unwrap();
    let settings = state.snapshot();
    let mut style = settings.global;
    style.crease_width = 3.;
    edit(
        &service,
        NprPlaygroundIntent::SetLook {
            style,
            layers: settings.style_layers,
        },
    )
    .unwrap();
    let effective = state.snapshot().objects["cube"].effective_style(state.snapshot().global);
    assert_eq!(effective.outline_width, 7.);
    assert_eq!(effective.crease_width, 3.);
    let renderer = NprPlaygroundRenderService::default();
    renderer.rebuild(&state.snapshot(), [256, 256]).unwrap();
    assert!(!renderer.snapshot().unwrap().packet.strokes.is_empty());
}

#[test]
fn layer_reordering_blend_and_sparse_overrides_are_undoable_and_paper_is_scene_owned() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let original = state.snapshot();
    let mut layers = original.style_layers.clone();
    let layer = layers.layer_mut("hatching").unwrap();
    layer.opacity = 0.3;
    layer.blend = NprBlendMode::Screen;
    layers.layers.rotate_right(1);
    edit(
        &service,
        NprPlaygroundIntent::SetLook {
            style: original.global,
            layers: layers.clone(),
        },
    )
    .unwrap();
    assert_eq!(state.snapshot().style_layers, layers);
    let mut object = state.snapshot().objects["cube"].clone();
    let mut local = layers.clone();
    local.layer_mut("fill").unwrap().opacity = 0.2;
    object.style_layer_overrides = NprStyleLayerOverrides::from_resolved(&layers, &local).unwrap();
    edit(
        &service,
        NprPlaygroundIntent::SetObject {
            object: "cube".into(),
            settings: object.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        state.snapshot().objects["cube"]
            .effective_layers(&layers)
            .layer("fill")
            .unwrap()
            .opacity,
        0.2
    );
    object.style_overrides = ComicInkOverrides::default();
    object.style_overrides.paper = Some(glam::Vec4::ZERO);
    assert!(
        edit(
            &service,
            NprPlaygroundIntent::SetObject {
                object: "cube".into(),
                settings: object
            }
        )
        .is_err()
    );
    edit(&service, NprPlaygroundIntent::Undo).unwrap();
    edit(&service, NprPlaygroundIntent::Undo).unwrap();
    assert_eq!(state.snapshot(), original);
}

#[test]
fn sidecar_reload_retains_inheritance_and_rejects_unknown_style_fields() {
    let mut settings = Settings::for_scene();
    settings
        .objects
        .get_mut("cube")
        .unwrap()
        .style_overrides
        .outline_width = Some(7.);
    let profile = NprSceneProfileDocument::from_settings(&settings, None).unwrap();
    let mut restored = profile
        .resolve(&BTreeMap::new(), &BTreeMap::new(), &Default::default())
        .unwrap();
    restored.global.crease_width = 3.;
    assert_eq!(
        restored.objects["cube"]
            .effective_style(restored.global)
            .outline_width,
        7.
    );
    assert_eq!(
        restored.objects["cube"]
            .effective_style(restored.global)
            .crease_width,
        3.
    );
    let bad = NprLookPatch {
        style: BTreeMap::from([("typo".into(), serde_json::json!(1))]),
        ..Default::default()
    };
    assert!(
        resolve_look(
            &BTreeMap::new(),
            &bad,
            None,
            &Default::default(),
            &Default::default(),
            &Default::default()
        )
        .unwrap_err()
        .contains("unknown look field")
    );
}

#[test]
fn sketch_pause_does_not_stop_model_playback() {
    let state = Arc::new(NprPlaygroundState::default());
    state.settings.lock().unwrap().objects.get_mut("cube").unwrap().rotating = true;
    let service = NprPlaygroundService::new(state.clone());
    edit(
        &service,
        NprPlaygroundIntent::SetMotion {
            paused: false,
            speed: 1.,
            sketch_paused: true,
        },
    )
    .unwrap();
    let before = state.snapshot().objects["cube"].rotation;
    state.tick(0.1);
    assert_ne!(state.snapshot().objects["cube"].rotation, before);
    assert!(state.snapshot().sketch_paused);
}

#[test]
fn explicit_pose_can_reset_animation_to_the_existing_authored_pose() {
    let state = Arc::new(NprPlaygroundState::default());
    state.settings.lock().unwrap().objects.get_mut("cube").unwrap().rotating = true;
    let service = NprPlaygroundService::new(state.clone());
    let authored = service.domain_snapshot().settings.objects["cube"].clone();
    state.tick(0.25);
    let live = state.snapshot().objects["cube"].rotation;
    assert_ne!(live, authored.rotation);
    let mut object = authored.clone();
    object.scale = 1.5;
    edit(
        &service,
        NprPlaygroundIntent::SetObject {
            object: "cube".into(),
            settings: object,
        },
    )
    .unwrap();
    assert_eq!(state.snapshot().objects["cube"].rotation, live);
    edit(
        &service,
        NprPlaygroundIntent::SetObjectPose {
            object: "cube".into(),
            position: authored.position,
            rotation: authored.rotation,
            scale: 1.5,
        },
    )
    .unwrap();
    assert_eq!(state.snapshot().objects["cube"].rotation, authored.rotation);
}
