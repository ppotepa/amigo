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

    let mut settings = Settings::for_scene(false);
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
fn authored_underpainting_medium_changes_only_the_paint_packet_channel() {
    use amigo_npr_playground_plugin::{NprPlaygroundRenderService, state::Settings};

    let mut settings = Settings::for_scene(false);
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
    assert!(
        command
            .packet
            .underpainting
            .iter()
            .all(|triangle| triangle.coverage == 0.0)
    );
    assert!(!command.packet.fills.is_empty());
}
