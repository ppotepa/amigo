use amigo_npr_playground_plugin::{
    NprPlaygroundState,
    documents::{NprLookDocument, NprLookPatch, NprSceneProfileDocument},
    playground::{NprPlaygroundIntent as Intent, NprPlaygroundService},
    state::Settings,
};
use amigo_playground_api::PlaygroundProvider;
use std::{fs, path::Path, sync::Arc};

fn fixture() -> (
    tempfile::TempDir,
    Arc<NprPlaygroundState>,
    NprPlaygroundService,
) {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("npr/looks")).unwrap();
    fs::write(
        root.path().join("npr/looks/base.npr-look.yml"),
        "id: base\nlook: {}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("npr/looks/drawing.npr-look.yml"),
        "id: drawing\nincludes: [base]\nlook: {}\n",
    )
    .unwrap();
    let mut profile =
        NprSceneProfileDocument::from_settings(&Settings::for_scene(), Some("drawing".into()))
            .unwrap();
    profile.look = NprLookPatch::default();
    fs::write(
        root.path().join("npr.scene.yml"),
        serde_yaml::to_string(&profile).unwrap(),
    )
    .unwrap();
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    service
        .open_scene(root.path(), Path::new("npr.scene.yml"))
        .unwrap();
    (root, state, service)
}

fn send(service: &NprPlaygroundService, intent: Intent) {
    service
        .dispatch_intent(1, service.revision(), "drawing-test".into(), intent)
        .unwrap();
}

#[test]
fn preview_is_transient_cancel_restores_and_apply_has_one_undo() {
    let (_root, state, service) = fixture();
    let before = service.domain_snapshot().settings;
    for i in 1..=8 {
        let mut value = before.style_layers.layer("contours").unwrap().clone();
        value.opacity = i as f32 / 10.0;
        send(
            &service,
            Intent::PreviewLayer {
                layer: value.id.clone(),
                value,
            },
        );
    }
    assert_eq!(service.domain_snapshot().settings, before);
    assert!(!service.domain_snapshot().dirty);
    assert!(!service.domain_snapshot().can_undo);
    assert_eq!(
        state
            .snapshot()
            .style_layers
            .layer("contours")
            .unwrap()
            .opacity,
        0.8
    );
    send(&service, Intent::CancelLayerPreview);
    assert_eq!(state.snapshot().style_layers, before.style_layers);
    assert!(!service.domain_snapshot().can_undo);
    let mut value = before.style_layers.layer("contours").unwrap().clone();
    value.opacity = 0.3;
    send(
        &service,
        Intent::PreviewLayer {
            layer: value.id.clone(),
            value: value.clone(),
        },
    );
    send(
        &service,
        Intent::ReplaceLayer {
            layer: value.id.clone(),
            value,
        },
    );
    assert!(service.domain_snapshot().look_dirty);
    assert!(service.domain_snapshot().preview_layer.is_none());
    send(&service, Intent::Undo);
    assert_eq!(service.domain_snapshot().settings, before);
    assert!(!service.domain_snapshot().can_undo);
}

#[test]
fn cancelled_connection_discards_preview_without_saving() {
    let (_root, state, service) = fixture();
    let before = service.domain_snapshot().settings;
    let mut value = before.style_layers.layer("contours").unwrap().clone();
    value.opacity = 0.0;
    send(
        &service,
        Intent::PreviewLayer {
            layer: value.id.clone(),
            value,
        },
    );
    service.cancel_interaction();
    assert_eq!(state.snapshot().style_layers, before.style_layers);
    assert!(service.domain_snapshot().preview_layer.is_none());
    assert!(!service.domain_snapshot().dirty);
}

#[test]
fn camera_does_not_dirty_preset_and_does_not_commit_line_preview() {
    let (_root, state, service) = fixture();
    let mut value = service
        .domain_snapshot()
        .settings
        .style_layers
        .layer("contours")
        .unwrap()
        .clone();
    value.opacity = 0.2;
    send(
        &service,
        Intent::PreviewLayer {
            layer: value.id.clone(),
            value,
        },
    );
    send(
        &service,
        Intent::Navigate {
            mode: "orbit".into(),
            dx: 20.0,
            dy: 5.0,
            wheel: 0.0,
            x: 0.0,
            y: 0.0,
            width: 512,
            height: 512,
            focused: true,
        },
    );
    assert!(!service.domain_snapshot().look_dirty);
    assert!(service.domain_snapshot().dirty);
    assert_eq!(
        state
            .snapshot()
            .style_layers
            .layer("contours")
            .unwrap()
            .opacity,
        0.2
    );
    send(&service, Intent::CancelLayerPreview);
    assert_eq!(
        state
            .snapshot()
            .style_layers
            .layer("contours")
            .unwrap()
            .opacity,
        1.0
    );
}

#[test]
fn deleted_inherited_lines_and_custom_appearance_survive_save_and_fresh_open() {
    let (root, _state, service) = fixture();
    send(
        &service,
        Intent::DeleteLayer {
            layer: "contours".into(),
        },
    );
    let mut value = service
        .domain_snapshot()
        .settings
        .style_layers
        .layer("creases")
        .unwrap()
        .clone();
    value.brush.as_mut().unwrap().width = Some(6.5);
    send(
        &service,
        Intent::SaveAppearance {
            layer: value.id.clone(),
            value,
            name: "My pen".into(),
            update_matching: false,
        },
    );
    let reference = service
        .domain_snapshot()
        .settings
        .style_layers
        .layer("creases")
        .unwrap()
        .brush
        .as_ref()
        .unwrap()
        .brush
        .clone();
    send(&service, Intent::SaveLook);
    assert!(!service.domain_snapshot().look_dirty);
    let saved: NprLookDocument = serde_yaml::from_slice(
        &fs::read(root.path().join("npr/looks/drawing.npr-look.yml")).unwrap(),
    )
    .unwrap();
    assert_eq!(saved.includes, vec!["base"]);
    assert!(saved.look.removed_layers.contains("contours"));
    assert_eq!(saved.look.brushes.resolve(&reference).unwrap().width, 6.5);
    let reopened = NprPlaygroundService::new(Arc::new(NprPlaygroundState::default()));
    reopened
        .open_scene(root.path(), Path::new("npr.scene.yml"))
        .unwrap();
    assert!(
        reopened
            .domain_snapshot()
            .settings
            .style_layers
            .layer("contours")
            .is_none()
    );
    assert_eq!(
        reopened
            .domain_snapshot()
            .brushes
            .resolve(&reference)
            .unwrap()
            .width,
        6.5
    );
    send(
        &service,
        Intent::SaveAsLook {
            id: "portable".into(),
        },
    );
    send(
        &reopened,
        Intent::UseLook {
            id: "portable".into(),
        },
    );
    assert_eq!(
        reopened.domain_snapshot().settings.style_layers,
        service.domain_snapshot().settings.style_layers
    );
    send(&reopened, Intent::SaveAll);
    send(&reopened, Intent::Reload);
    assert_eq!(
        reopened.domain_snapshot().active_look.as_deref(),
        Some("portable")
    );
    assert!(
        reopened
            .domain_snapshot()
            .settings
            .style_layers
            .layer("contours")
            .is_none()
    );
}

#[test]
fn save_appearance_updates_matching_references_atomically_and_respects_locks() {
    let (_root, _state, service) = fixture();
    let before = service.domain_snapshot().settings;
    let mut value = before.style_layers.layer("contours").unwrap().clone();
    value.brush.as_mut().unwrap().width = Some(4.0);
    send(
        &service,
        Intent::SetLayerLock {
            layer: "form-lines".into(),
            locked: true,
        },
    );
    assert!(
        service
            .dispatch_intent(
                1,
                service.revision(),
                "appearance".into(),
                Intent::SaveAppearance {
                    layer: value.id.clone(),
                    value: value.clone(),
                    name: "Ink liner".into(),
                    update_matching: true,
                }
            )
            .is_err()
    );
    assert_eq!(service.domain_snapshot().settings, before);
    send(
        &service,
        Intent::SetLayerLock {
            layer: "form-lines".into(),
            locked: false,
        },
    );
    send(
        &service,
        Intent::SaveAppearance {
            layer: value.id.clone(),
            value,
            name: "Ink liner".into(),
            update_matching: true,
        },
    );
    let after = service.domain_snapshot().settings;
    let contour = &after
        .style_layers
        .layer("contours")
        .unwrap()
        .brush
        .as_ref()
        .unwrap()
        .brush;
    let form = &after
        .style_layers
        .layer("form-lines")
        .unwrap()
        .brush
        .as_ref()
        .unwrap()
        .brush;
    assert_eq!(contour, form);
    assert_eq!(contour.version, 2);
    assert_eq!(after.brushes.resolve(contour).unwrap().width, 4.0);
    assert!(
        after
            .style_layers
            .layer("contours")
            .unwrap()
            .brush
            .as_ref()
            .unwrap()
            .width
            .is_none()
    );
    send(&service, Intent::Undo);
    assert_eq!(service.domain_snapshot().settings, before);
    assert!(!service.domain_snapshot().can_undo);
}

#[test]
fn saving_an_old_pinned_appearance_allocates_the_next_revision_on_backend() {
    let (_root, _state, service) = fixture();
    let old = service
        .domain_snapshot()
        .settings
        .style_layers
        .layer("contours")
        .unwrap()
        .clone();
    for expected in [2, 3] {
        send(
            &service,
            Intent::SaveAppearance {
                layer: old.id.clone(),
                value: old.clone(),
                name: "Ink liner".into(),
                update_matching: false,
            },
        );
        assert_eq!(
            service
                .domain_snapshot()
                .settings
                .style_layers
                .layer("contours")
                .unwrap()
                .brush
                .as_ref()
                .unwrap()
                .brush
                .version,
            expected
        );
    }
}

#[test]
fn targets_and_model_colour_use_the_public_wire_contract() {
    use amigo_render_npr::{GeometryTarget, NprGeometrySource, NprLayerColorSource};
    let target = GeometryTarget::SurfaceFeatures {
        features: vec!["shadow-hatch".into()],
    };
    assert!(target.includes("cube", NprGeometrySource::ShadowHatch));
    assert!(!target.includes("cube", NprGeometrySource::Silhouette));
    assert_eq!(
        serde_json::to_value(NprLayerColorSource::ModelBaseColor).unwrap()["kind"],
        "model-base-color"
    );
    let (_root, _state, service) = fixture();
    let mut value = service
        .domain_snapshot()
        .settings
        .style_layers
        .layer("contours")
        .unwrap()
        .clone();
    value.color_source = NprLayerColorSource::ModelBaseColor;
    send(
        &service,
        Intent::ReplaceLayer {
            layer: value.id.clone(),
            value,
        },
    );
}
