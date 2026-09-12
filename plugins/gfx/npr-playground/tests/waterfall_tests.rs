#[test]
fn manifest_and_service_use_typed_npr_contract() {
    let service = amigo_npr_playground_plugin::NprPlaygroundRenderService::default();
    service.rebuild_cube([512, 512], 7);
    assert_eq!(service.snapshot().unwrap().packet.stats.geometry, 1);
}

#[test]
fn scene_layer_intent_reaches_the_extracted_npr_draw_command() {
    use amigo_npr_playground_plugin::{NprPlaygroundRenderService, state::Settings};
    use amigo_render_npr::{NprBlendMode, NprLayerColorSource};

    let mut settings = Settings::for_scene();
    let hatching = settings
        .style_layers
        .layers
        .iter_mut()
        .find(|layer| layer.id == "hatching")
        .unwrap();
    hatching.opacity = 0.31;
    hatching.blend = NprBlendMode::Screen;
    hatching.color_source = NprLayerColorSource::ModelBaseColor;

    let service = NprPlaygroundRenderService::default();
    service.rebuild(&settings, [512, 512]).unwrap();
    let commands = service.commands();
    let hatching = commands[0].layers.layer("hatching").unwrap();
    assert_eq!(hatching.opacity, 0.31);
    assert_eq!(hatching.blend, NprBlendMode::Screen);
    assert_eq!(hatching.color_source, NprLayerColorSource::ModelBaseColor);
    assert_eq!(
        commands[0].material_base_color,
        Some(
            settings.objects[&settings.selected]
                .material_base_color
                .to_array()
        )
    );
}

#[test]
fn authored_underpainting_medium_is_kept_as_per_layer_compositor_state() {
    use amigo_npr_playground_plugin::{NprPlaygroundRenderService, state::Settings};

    let mut settings = Settings::for_scene();
    settings
        .style_layers
        .layers
        .iter_mut()
        .find(|layer| layer.id == "underpainting")
        .unwrap()
        .paint
        .as_mut()
        .unwrap()
        .wash = 0.0;
    let service = NprPlaygroundRenderService::default();
    service.rebuild(&settings, [512, 512]).unwrap();
    let mut commands = service.commands();
    let command = commands.remove(0);
    assert!(!command.packet.underpainting.is_empty());
    // Source geometry is shared by every wash layer.  Applying wash here
    // would make another wash inherit this layer's zero coverage; the WGPU
    // compositor applies this medium only while drawing its owning layer.
    assert!(
        command
            .packet
            .underpainting
            .iter()
            .any(|triangle| triangle.coverage > 0.0)
    );
    assert_eq!(
        command
            .layers
            .layer("underpainting")
            .unwrap()
            .paint
            .unwrap()
            .wash,
        0.0
    );
    assert!(!command.packet.fills.is_empty());
}

#[test]
fn two_contour_layers_emit_two_independent_layer_contributions() {
    use amigo_npr_playground_plugin::{NprPlaygroundRenderService, state::Settings};

    let mut settings = Settings::for_scene();
    let mut soft_contour = settings.style_layers.layer("contours").unwrap().clone();
    soft_contour.id = "contours-soft".into();
    soft_contour.label = "Soft contour".into();
    soft_contour.opacity = 0.35;
    settings.style_layers.layers.push(soft_contour);

    let service = NprPlaygroundRenderService::default();
    service.rebuild(&settings, [512, 512]).unwrap();
    let command = service.commands().remove(0);
    let contour_layers = command
        .packet
        .strokes
        .iter()
        .filter(|stroke| {
            stroke.role == amigo_render_npr::StrokeRole::Feature
                && stroke.class == amigo_render_npr::FeatureClass::Silhouette
        })
        .filter_map(|stroke| stroke.layer_id.as_deref())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        contour_layers,
        std::collections::BTreeSet::from(["contours", "contours-soft"])
    );
}

use amigo_npr_playground_plugin::{NprPlaygroundRenderService, state::Settings};

#[test]
fn shared_surfaces_preserve_independent_camera_packets_and_temporal_histories() {
    let renderer = NprPlaygroundRenderService::default();
    let companion = renderer.fork_view();
    let mut settings = Settings::for_scene();
    settings.sketch_paused = true;
    renderer
        .rebuild_with_delta(&settings, [640, 360], 0.016)
        .unwrap();
    let original = renderer.commands();
    let mut second_camera = settings.clone();
    second_camera.camera_yaw += 20.0;
    companion
        .rebuild_with_delta(&second_camera, [960, 540], 0.016)
        .unwrap();
    assert_eq!(renderer.commands(), original);
    assert_ne!(companion.commands(), original);
    assert_eq!(renderer.stats()["packet_builds"], 1);
    for _ in 0..20 {
        renderer
            .rebuild_with_delta(&settings, [640, 360], 0.016)
            .unwrap();
    }
    assert_eq!(renderer.stats()["packet_builds"], 1);
    assert_eq!(companion.stats()["packet_builds"], 1);
}
