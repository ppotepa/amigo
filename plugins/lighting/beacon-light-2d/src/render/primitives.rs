use amigo_render_api::{
    BeaconLight2dPrimitive, LayeredImageViewportFit2dPrimitive, LightContributionKind2d,
    LightEmitterKind2d, LightSource2dCommon, LightSource2dCommonParams, RenderContribution2d,
    RenderPrimitive2d, Renderable2dCommon, Renderable2dItem, Renderable2dKind,
};

use crate::BeaconLight2dDrawCommand;

pub fn beacon_draw_command_to_render_primitive(
    command: &BeaconLight2dDrawCommand,
) -> RenderPrimitive2d {
    RenderPrimitive2d::RadialLightVisual(BeaconLight2dPrimitive {
        center: command.center,
        color: command.color,
        intensity: command.intensity,
        pulse: command.pulse,
        core_radius_px: command.core_radius_px,
        halo_radius_px: command.halo_radius_px,
        glow_strength: command.glow_strength,
        rotation_radians: command.rotation_radians,
        beam_enabled: command.beam_enabled,
        beam_length_px: command.beam_length_px,
        beam_width_degrees: command.beam_width_degrees,
        beam_strength: command.beam_strength,
        aberration_px: command.aberration_px,
        bloom: command.bloom,
        camera_intensity: command.camera_response.intensity,
        camera_glare: command.camera_response.glare,
        overlay_visible: command
            .render_contributions
            .enabled_or(amigo_render_api::render_contribution_roles::OVERLAY_VISIBLE, true),
        distance_m: command.distance_m,
        z_depth: command.z_depth,
        viewport_fit: match command.viewport_fit {
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Fixed => {
                LayeredImageViewportFit2dPrimitive::Fixed
            }
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Stretch => {
                LayeredImageViewportFit2dPrimitive::Stretch
            }
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Contain => {
                LayeredImageViewportFit2dPrimitive::Contain
            }
            amigo_scene::LayeredImageViewportFit2dSceneCommand::Cover => {
                LayeredImageViewportFit2dPrimitive::Cover
            }
        },
        viewport_canvas_size: command.viewport_canvas_size,
    })
}

pub fn beacon_draw_command_to_renderable_2d(
    command: &BeaconLight2dDrawCommand,
) -> Renderable2dItem {
    Renderable2dItem::new(
        Renderable2dCommon::world(
            command.entity_name.clone(),
            "BeaconLight2D",
            command.render_layer.clone(),
            command.z_index,
            Renderable2dKind::Beacon,
        ),
        beacon_draw_command_to_render_primitive(command),
    )
}

pub fn beacon_draw_command_to_light_contribution(
    command: &BeaconLight2dDrawCommand,
) -> RenderContribution2d {
    let mut contributions = Vec::new();
    if command.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::RELIGHT_PLATE,
        true,
    ) {
        contributions.push(LightContributionKind2d::RelightPlate);
    }
    if command.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
        true,
    ) {
        contributions.push(LightContributionKind2d::BloomSource);
    }
    if command.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
        true,
    ) {
        contributions.push(LightContributionKind2d::CameraFxSource);
    }

    let reason = if contributions.is_empty() {
        "all_light_roles_disabled"
    } else {
        "active_light_emitter"
    };
    let params = LightSource2dCommonParams {
        owner: command.entity_name.clone(),
        component_kind: "BeaconLight2D".to_owned(),
        emitter_kind: LightEmitterKind2d::Beacon,
        emitter_id: None,
        render_layer: Some(command.render_layer.clone()),
        color_rgba: Some([command.color.r, command.color.g, command.color.b, command.color.a]),
        intensity: Some(command.intensity),
        effective_intensity: Some(command.intensity * command.color.a),
        response: Some(1.0),
        camera_response: Some(command.camera_response),
        bloom: Some(command.bloom),
        radius_px: Some(command.halo_radius_px.max(command.core_radius_px)),
        falloff: None,
        distance_m: command.distance_m,
        z_depth: command.z_depth,
        contributions,
        reason: reason.to_owned(),
        position_px: Some([command.center.x, command.center.y]),
    };

    let source = if params.contributions.is_empty() {
        LightSource2dCommon::skipped(params)
    } else {
        LightSource2dCommon::active(params)
    };
    RenderContribution2d::LightSource2d(source)
}
