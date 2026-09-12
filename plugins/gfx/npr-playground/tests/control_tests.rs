use amigo_assets::{AssetBrowserEntryState, AssetCatalog, AssetSourceId};
use amigo_npr_playground_plugin::{NprPlaygroundRenderService, NprPlaygroundState, state::MODELS};
use amigo_npr_playground_plugin::{
    asset_browser::register_models,
    scene::NprPlaygroundSceneDocument,
    state::{ConstructionAnchorSettings, ConstructionMarkSettings},
};
use amigo_render_npr::{NprSurfaceIntent, NprSurfaceMode, StrokeRole};
use glam::Vec2;
use std::{collections::BTreeMap, sync::Arc};

#[test]
fn surface_pick_returns_a_source_anchor_for_the_selected_model() {
    let state = NprPlaygroundState::default();
    let settings = state.snapshot();
    let render = NprPlaygroundRenderService::default();

    let pick = render
        .pick_surface(&settings, [512, 512], Vec2::new(256.0, 256.0))
        .expect("the centered camera ray should hit the selected cube");

    assert_eq!(pick.object_id, "cube");
    assert!(pick.position.is_finite());
    assert!(pick.normal.is_finite());
    assert!(pick.anchor.triangle < 12);
    assert!((pick.anchor.barycentric.iter().sum::<f32>() - 1.0).abs() < 1e-5);
}

#[test]
fn construction_authoring_commits_open_and_closed_source_lines() {
    let state = NprPlaygroundState::default();
    state.begin_construction_mark().unwrap();
    assert!(state.construction_authoring_active());

    state
        .place_construction_anchor(
            "cube",
            ConstructionAnchorSettings {
                triangle: 0,
                barycentric: [0.7, 0.2, 0.1],
            },
        )
        .unwrap();
    assert!(
        state.snapshot().objects["cube"]
            .construction_marks
            .is_empty()
    );
    assert!(state.commit_construction_mark(false).is_err());

    state
        .place_construction_anchor(
            "cube",
            ConstructionAnchorSettings {
                triangle: 0,
                barycentric: [0.1, 0.7, 0.2],
            },
        )
        .unwrap();
    state.commit_construction_mark(false).unwrap();
    let marks = &state.snapshot().objects["cube"].construction_marks;
    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0].anchors.len(), 2);
    assert!(!state.construction_authoring_active());

    state.begin_construction_mark().unwrap();
    for barycentric in [[0.7, 0.2, 0.1], [0.1, 0.7, 0.2], [0.2, 0.1, 0.7]] {
        state
            .place_construction_anchor(
                "cube",
                ConstructionAnchorSettings {
                    triangle: 0,
                    barycentric,
                },
            )
            .unwrap();
    }
    state.commit_construction_mark(true).unwrap();
    let marks = &state.snapshot().objects["cube"].construction_marks;
    assert_eq!(marks.len(), 2);
    assert!(marks[1].closed);
    assert_eq!(marks[1].anchors.len(), 3);

    state.select_construction_mark(-1).unwrap();
    state.delete_selected_construction_mark().unwrap();
    let remaining = &state.snapshot().objects["cube"].construction_marks;
    assert_eq!(remaining.len(), 1);
    assert!(remaining[0].closed);

    let document = state.authored_scene_document().unwrap();
    assert_eq!(document.objects["cube"].construction_marks.len(), 1);
    let restored = document
        .resolve(&BTreeMap::new(), &BTreeMap::new(), &Default::default())
        .unwrap();
    assert_eq!(
        restored.objects["cube"].construction_marks,
        state.snapshot().objects["cube"].construction_marks
    );
}

#[test]
fn construction_authoring_waits_for_the_panel_click_to_be_released() {
    let state = NprPlaygroundState::default();
    state.begin_construction_mark().unwrap();
    assert!(!state.construction_authoring_accepts_click(true));
    assert!(!state.construction_authoring_accepts_click(false));
    assert!(state.construction_authoring_accepts_click(false));
}

#[test]
fn construction_authoring_can_remove_its_latest_draft_point() {
    let state = NprPlaygroundState::default();
    state.begin_construction_mark().unwrap();
    for barycentric in [[0.7, 0.2, 0.1], [0.1, 0.7, 0.2]] {
        state
            .place_construction_anchor(
                "cube",
                ConstructionAnchorSettings {
                    triangle: 0,
                    barycentric,
                },
            )
            .unwrap();
    }
    assert_eq!(
        state.render_snapshot().objects["cube"]
            .construction_marks
            .len(),
        1
    );
    state.undo_construction_anchor().unwrap();
    assert!(
        state.render_snapshot().objects["cube"]
            .construction_marks
            .is_empty()
    );
    state.undo_construction_anchor().unwrap();
    assert!(state.undo_construction_anchor().is_err());
}

#[test]
fn construction_authoring_renders_a_transient_preview_without_serializing_it() {
    let state = NprPlaygroundState::default();
    state.begin_construction_mark().unwrap();
    for barycentric in [[0.7, 0.2, 0.1], [0.1, 0.7, 0.2]] {
        state
            .place_construction_anchor(
                "cube",
                ConstructionAnchorSettings {
                    triangle: 0,
                    barycentric,
                },
            )
            .unwrap();
    }

    assert!(
        state.snapshot().objects["cube"]
            .construction_marks
            .is_empty()
    );
    let preview = state.render_snapshot();
    let marks = &preview.objects["cube"].construction_marks;
    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0].id, u32::MAX);
    assert_eq!(marks[0].anchors.len(), 2);
    assert!(!marks[0].closed);
    let render = NprPlaygroundRenderService::default();
    render.rebuild(&preview, [512, 512]).unwrap();
    assert_eq!(render.commands()[0].packet.stats.construction_marks, 1);
    assert!(
        state
            .authored_scene_document()
            .unwrap()
            .objects
            .values()
            .all(|object| object.construction_marks.is_empty())
    );
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
fn authored_construction_marks_flow_from_object_state_to_render_packet() {
    let state = NprPlaygroundState::default();
    state
        .settings
        .lock()
        .unwrap()
        .objects
        .get_mut("cube")
        .unwrap()
        .construction_marks = vec![ConstructionMarkSettings {
        id: 0x4000_0100,
        anchors: vec![
            ConstructionAnchorSettings {
                triangle: 0,
                barycentric: [0.70, 0.20, 0.10],
            },
            ConstructionAnchorSettings {
                triangle: 0,
                barycentric: [0.10, 0.70, 0.20],
            },
        ],
        closed: false,
        width_scale: 0.5,
        opacity: 0.35,
    }];
    let render = NprPlaygroundRenderService::default();
    render.rebuild(&state.snapshot(), [512, 512]).unwrap();
    let packet = &render.commands()[0].packet;
    assert_eq!(packet.stats.construction_marks, 1);
    assert!(
        packet
            .strokes
            .iter()
            .any(|stroke| stroke.role == StrokeRole::Construction)
    );
}

#[test]
fn authored_construction_marks_reject_invalid_geometry_before_extraction() {
    let mut settings = NprPlaygroundState::default().snapshot();
    settings.objects.get_mut("cube").unwrap().construction_marks = vec![ConstructionMarkSettings {
        id: 0x4000_0101,
        anchors: vec![ConstructionAnchorSettings {
            triangle: 0,
            barycentric: [0.8, 0.8, 0.8],
        }],
        closed: false,
        width_scale: 0.5,
        opacity: 0.35,
    }];

    assert!(settings.validate().is_err());
}

#[test]
fn authored_sidecar_preserves_camera_surface_and_object_intent() {
    let state = NprPlaygroundState::default();
    let mut settings = amigo_npr_playground_plugin::state::Settings::for_scene(true);
    settings.seed = 99;
    settings.camera_distance = 18.;
    settings.camera_yaw = 31.;
    let cube = settings.objects.get_mut("cube").unwrap();
    cube.rotating = false;
    cube.surface_subdivision_level = 2;
    cube.smooth_weld_relative_tolerance = 0.000_02;
    let profile = amigo_npr_playground_plugin::documents::NprSceneProfileDocument::from_settings(
        &settings, None,
    )
    .unwrap();
    state.apply_authored_scene(profile).unwrap();
    let restored = state.snapshot();
    assert_eq!(restored.seed, 99);
    assert_eq!(restored.camera_distance, 18.);
    assert_eq!(restored.camera_yaw, 31.);
    assert!(!restored.objects["cube"].rotating);
    assert_eq!(restored.objects["cube"].surface_subdivision_level, 2);
    assert_eq!(
        restored.objects["cube"].smooth_weld_relative_tolerance,
        0.000_02
    );
}

#[test]
fn gallery_scene_file_starts_empty_and_hydrates_no_model_instances() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../mods/npr-playground");
    let scene: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(root.join("scenes/gallery/scene.yml")).unwrap(),
    )
    .unwrap();
    let component = scene["entities"]
        .as_sequence()
        .unwrap()
        .iter()
        .flat_map(|entity| entity["components"].as_sequence().unwrap())
        .find(|component| component["type"] == "amigo.gfx.npr-playground.NprSettings")
        .unwrap();
    let mut payload = component.as_mapping().unwrap().clone();
    payload.remove(serde_yaml::Value::String("type".into()));
    let authored: NprPlaygroundSceneDocument =
        serde_yaml::from_value(serde_yaml::Value::Mapping(payload)).unwrap();

    assert_eq!(authored.profile, "npr.scene.yml");
    let state = Arc::new(NprPlaygroundState::default());
    let service = amigo_npr_playground_plugin::playground::NprPlaygroundService::new(state.clone());
    service
        .open_scene(&root, std::path::Path::new("scenes/gallery/npr.scene.yml"))
        .unwrap();
    assert!(state.snapshot().objects.is_empty());
    assert!(state.snapshot().selected.is_empty());
}

#[test]
fn blank_scene_extracts_paper_without_any_model_draw_command() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../mods/npr-playground");
    let render = NprPlaygroundRenderService::default();
    render.load_models(&root).unwrap();
    let settings = amigo_npr_playground_plugin::state::Settings::empty_scene(true);

    render.rebuild(&settings, [512, 512]).unwrap();
    assert!(render.commands().is_empty());
    assert_eq!(
        render.background().unwrap().color,
        settings.global.paper.to_array()
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
fn asset_browser_models_are_ready_and_route_add_and_replace_to_npr_state() {
    let catalog = AssetCatalog::default();
    register_models(&catalog, std::path::Path::new("mods/npr-playground"));
    let source = catalog
        .asset_source_snapshot(&AssetSourceId::from_kind(
            &amigo_assets::AssetSourceKind::Mod("npr-playground".into()),
        ))
        .unwrap();
    assert_eq!(source.entries.len(), 2);
    assert_eq!(
        catalog
            .asset_source_snapshot(&AssetSourceId::from_kind(
                &amigo_assets::AssetSourceKind::Engine
            ))
            .unwrap()
            .entries
            .len(),
        4
    );
    assert!(
        source
            .entries
            .iter()
            .all(|entry| matches!(entry.state, AssetBrowserEntryState::Ready))
    );

    let state = Arc::new(NprPlaygroundState::default());
    *state.settings.lock().unwrap() =
        amigo_npr_playground_plugin::state::Settings::empty_scene(true);
    let service = amigo_npr_playground_plugin::playground::NprPlaygroundService::new(state.clone());
    service
        .dispatch_intent(
            1,
            0,
            "models".into(),
            amigo_npr_playground_plugin::playground::NprPlaygroundIntent::SelectModel {
                model: "cube".into(),
            },
        )
        .unwrap();
    let selected = state.snapshot().selected;
    assert_eq!(selected, "cube");
    assert_eq!(state.snapshot().objects[&selected].model, "cube");

    let mut replacement = state.snapshot().objects[&selected].clone();
    replacement.model = "suzanne".into();
    service
        .dispatch_intent(
            2,
            1,
            "models".into(),
            amigo_npr_playground_plugin::playground::NprPlaygroundIntent::SetObject {
                object: selected.clone(),
                settings: replacement,
            },
        )
        .unwrap();
    assert_eq!(state.snapshot().objects[&selected].model, "suzanne");
}

#[test]
fn model_explorer_renders_one_selected_model_and_exposes_studio_tabs() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../mods/npr-playground");
    let state = Arc::new(NprPlaygroundState::default());
    state.configure_scene(false);
    let metadata = amigo_npr_playground_plugin::playground::NprPlaygroundService::metadata();
    assert_eq!(
        metadata["tabs"],
        serde_json::json!([
            "Scene",
            "Model",
            "Look",
            "Layers",
            "Camera",
            "Motion",
            "Diagnostics"
        ])
    );
    let render = NprPlaygroundRenderService::default();
    render.load_models(&root).unwrap();
    render.rebuild(&state.snapshot(), [1024, 768]).unwrap();
    assert_eq!(render.commands().len(), 1);
    assert!(!render.commands()[0].packet.fills.is_empty());
    assert!(!render.commands()[0].packet.strokes.is_empty());
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

#[test]
fn surface_intent_controls_the_extracted_proxy_not_just_panel_metadata() {
    use amigo_npr_playground_plugin::state::Settings;

    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../mods/npr-playground");
    let render = NprPlaygroundRenderService::default();
    render.load_models(&root).unwrap();

    let mut organic = Settings::for_scene(false);
    let cube = organic.objects.get_mut("cube").unwrap();
    cube.surface_intent = NprSurfaceIntent::Organic;
    cube.surface_mode = NprSurfaceMode::Polygonal;
    cube.surface_subdivision_level = 0;
    cube.style_overrides.smooth_draw_creases = Some(true);
    render.rebuild(&organic, [512, 512]).unwrap();
    let organic_packet = &render.commands()[0].packet;
    assert!(
        organic_packet.stats.surface_proxy_vertices > organic_packet.stats.surface_source_vertices,
        "Organic must select a prepared Smooth proxy even if a stale mode says Polygonal"
    );

    let mut hard_surface = organic;
    let cube = hard_surface.objects.get_mut("cube").unwrap();
    cube.surface_intent = NprSurfaceIntent::HardSurface;
    cube.surface_mode = NprSurfaceMode::Smooth;
    cube.surface_subdivision_level = 2;
    render.rebuild(&hard_surface, [512, 512]).unwrap();
    let hard_packet = &render.commands()[0].packet;
    assert_eq!(
        hard_packet.stats.surface_proxy_vertices, hard_packet.stats.surface_source_vertices,
        "HardSurface must preserve literal topology even if a stale mode says Smooth"
    );
}

#[test]
fn scene_component_requires_a_sidecar_reference_and_rejects_inline_settings() {
    let reference: NprPlaygroundSceneDocument =
        serde_yaml::from_str("profile: npr.scene.yml").unwrap();
    assert_eq!(reference.profile, "npr.scene.yml");
    assert!(serde_yaml::from_str::<NprPlaygroundSceneDocument>("gallery: true").is_err());
    assert!(
        serde_yaml::from_str::<NprPlaygroundSceneDocument>("profile: npr.scene.yml\ncamera: {}")
            .is_err()
    );
}

#[test]
fn single_model_navigation_keeps_the_selected_camera_fit() {
    let state = NprPlaygroundState::default();
    state.select_scene_object(-1).unwrap();
    assert_eq!(state.snapshot().selected, "cube");
    assert!(state.snapshot().camera_distance > 0.1);
    state.select_scene_object(1).unwrap();
    assert_eq!(state.snapshot().selected, "cube");

    state.configure_scene(false);
    let camera = state.snapshot().camera_distance;
    state.select_scene_object(1).unwrap();
    assert_eq!(state.snapshot().selected, "cube");
    assert!(state.snapshot().camera_distance > 0.1);
}
