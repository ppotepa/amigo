use crate::renderer::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct FocusBlurZDepthLayer {
    pub(super) layer_id: String,
    pub(super) order: f32,
    pub(super) z_depth: f32,
    pub(super) base_z_depth: f32,
    pub(super) effective_z_depth: f32,
    pub(super) distance_m: Option<f32>,
    pub(super) effective_distance_m: Option<f32>,
    pub(super) blur_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct FocusBlurLayerPlan {
    pub(super) depth_map_layers: BTreeSet<String>,
    pub(super) z_depth_layers: Vec<FocusBlurZDepthLayer>,
    pub(super) overlay_layers: BTreeSet<String>,
    pub(super) implicit_affected_layers: Option<BTreeSet<String>>,
    pub(super) has_explicit_render_depth: bool,
}

pub(super) fn focus_blur_effect_for(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
    host_id: &amigo_render_api::PostFxHost2dId,
    effect_id: &amigo_render_api::PostFx2dId,
) -> Option<amigo_render_api::FocusBlur2d> {
    stacks
        .iter()
        .find(|stack| &stack.host_id == host_id)
        .and_then(|stack| stack.effects.iter().find(|effect| &effect.id == effect_id))
        .and_then(|instance| instance.effect.clone().into_focus_blur())
}

pub(super) fn depth_debug_post_fx_for(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
    host_id: &amigo_render_api::PostFxHost2dId,
    effect_id: &amigo_render_api::PostFx2dId,
) -> Option<amigo_render_api::FocusBlur2d> {
    let effect = focus_blur_effect_for(stacks, host_id, effect_id)?;
    amigo_render_api::post_fx_focus_blur(effect.clone())
        .render_descriptor()
        .debug_policy
        .supports_depth_debug_view
        .then_some(effect)
}

pub(super) fn shutter_blur_effect_for(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
    host_id: &amigo_render_api::PostFxHost2dId,
    effect_id: &amigo_render_api::PostFx2dId,
) -> Option<amigo_render_api::ShutterBlur2d> {
    stacks
        .iter()
        .find(|stack| &stack.host_id == host_id)
        .and_then(|stack| stack.effects.iter().find(|effect| &effect.id == effect_id))
        .and_then(|instance| instance.effect.clone().into_shutter_blur())
}

pub(super) fn motion_debug_post_fx_for(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
    host_id: &amigo_render_api::PostFxHost2dId,
    effect_id: &amigo_render_api::PostFx2dId,
) -> Option<amigo_render_api::ShutterBlur2d> {
    let effect = shutter_blur_effect_for(stacks, host_id, effect_id)?;
    effect.is_active().then_some(effect)
}

pub(super) fn replay_scoped_layers_plan_for_effect(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
    render_layers: &[RenderLayer2dCommand],
    capture_input: Option<&amigo_render_api::CameraCaptureInput2d>,
    host_id: &amigo_render_api::PostFxHost2dId,
    effect_id: &amigo_render_api::PostFx2dId,
) -> Option<FocusBlurLayerPlan> {
    let effect = focus_blur_effect_for(stacks, host_id, effect_id)?;
    (amigo_render_api::post_fx_focus_blur(effect.clone())
        .render_descriptor()
        .output
        == amigo_render_api::PostFxRenderOutput::ReplayScopedLayers)
        .then(|| build_focus_blur_layer_plan(effect, render_layers, capture_input))
}

pub(super) fn focus_blur_layer_plan(
    stacks: &[amigo_render_api::ScopedPostFx2dStack],
    render_layers: &[RenderLayer2dCommand],
    capture_input: Option<&amigo_render_api::CameraCaptureInput2d>,
) -> Option<FocusBlurLayerPlan> {
    let effect = stacks.iter().find_map(|stack| {
        stack
            .effects
            .iter()
            .find_map(|instance| instance.effect.clone().into_focus_blur())
    })?;
    Some(build_focus_blur_layer_plan(
        effect,
        render_layers,
        capture_input,
    ))
}

pub(super) fn build_focus_blur_layer_plan(
    effect: amigo_render_api::FocusBlur2d,
    render_layers: &[RenderLayer2dCommand],
    capture_input: Option<&amigo_render_api::CameraCaptureInput2d>,
) -> FocusBlurLayerPlan {
    let has_explicit_render_depth = render_layers
        .iter()
        .any(|layer| !layer.depth.is_depth_map());
    let implicit_affected_layers = (!has_explicit_render_depth
        && !effect.affected_layers.is_empty())
    .then(|| effect.affected_layers.into_iter().collect::<BTreeSet<_>>());

    let mut depth_map_layers = BTreeSet::new();
    let mut z_depth_layers = Vec::new();
    let mut overlay_layers = BTreeSet::new();

    for layer in render_layers {
        match layer.depth.mode {
            amigo_render_api::RenderDepthMode2d::DepthMap => {
                depth_map_layers.insert(layer.id.clone());
            }
            amigo_render_api::RenderDepthMode2d::Distance
            | amigo_render_api::RenderDepthMode2d::ZDepth => {
                let capture_layer = capture_input.and_then(|input| {
                    input
                        .layers
                        .iter()
                        .find(|candidate| candidate.layer_id == layer.id)
                });
                let base_z_depth = capture_layer
                    .map(|capture| capture.base_z_depth)
                    .unwrap_or(layer.depth.z_depth)
                    .clamp(0.0, 1.0);
                let effective_z_depth = capture_layer
                    .map(|capture| capture.effective_z_depth)
                    .unwrap_or(layer.depth.z_depth)
                    .clamp(0.0, 1.0);
                z_depth_layers.push(FocusBlurZDepthLayer {
                    layer_id: layer.id.clone(),
                    order: layer.order,
                    z_depth: effective_z_depth,
                    base_z_depth,
                    effective_z_depth,
                    distance_m: layer.depth.distance_m,
                    effective_distance_m: capture_layer
                        .and_then(|capture| capture.effective_distance_m),
                    blur_scale: layer.depth.blur_scale,
                });
            }
            amigo_render_api::RenderDepthMode2d::Infinity => {
                z_depth_layers.push(FocusBlurZDepthLayer {
                    layer_id: layer.id.clone(),
                    order: layer.order,
                    z_depth: 0.0,
                    base_z_depth: 0.0,
                    effective_z_depth: 0.0,
                    distance_m: layer.depth.distance_m,
                    effective_distance_m: None,
                    blur_scale: layer.depth.blur_scale,
                });
            }
            amigo_render_api::RenderDepthMode2d::Overlay => {
                overlay_layers.insert(layer.id.clone());
            }
        }
    }
    z_depth_layers.sort_by(|left, right| {
        left.order
            .partial_cmp(&right.order)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.layer_id.cmp(&right.layer_id))
    });

    FocusBlurLayerPlan {
        depth_map_layers,
        z_depth_layers,
        overlay_layers,
        implicit_affected_layers,
        has_explicit_render_depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(id: &str, order: f32) -> RenderLayer2dCommand {
        RenderLayer2dCommand {
            source_mod: "test".to_owned(),
            id: id.to_owned(),
            label: None,
            order,
            visible: true,
            opacity: 1.0,
            depth: amigo_render_api::RenderDepth2d {
                mode: amigo_render_api::RenderDepthMode2d::Distance,
                distance_m: Some(1.0),
                z_depth: 0.5,
                blur_scale: 1.0,
            },
            optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
        }
    }

    #[test]
    fn focus_blur_plan_sorts_z_depth_layers_by_render_layer_order() {
        let plan = build_focus_blur_layer_plan(
            amigo_render_api::FocusBlur2d::default(),
            &[
                layer("weather.rain.super_near", 60.0),
                layer("title.depth2d", 20.0),
                layer("weather.rain.1m", 35.0),
            ],
            None,
        );

        let layer_ids = plan
            .z_depth_layers
            .iter()
            .map(|layer| layer.layer_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            layer_ids,
            vec![
                "title.depth2d",
                "weather.rain.1m",
                "weather.rain.super_near"
            ]
        );
    }

    #[test]
    fn focus_blur_plan_uses_effective_capture_layer_depth() {
        let mut capture_input = amigo_render_api::CameraCaptureInput2d::world_color(
            amigo_2d_spatial::DepthSpace2d::default(),
            Vec::new(),
        );
        capture_input
            .layers
            .push(amigo_render_api::ResolvedLayerOptics2d {
                layer_id: "weather.rain.mid".to_owned(),
                role: amigo_2d_spatial::OpticalLayerRole2d::SceneMedium,
                depth_mode: "distance".to_owned(),
                distance_m: Some(75.0),
                z_depth: 0.33,
                base_z_depth: 0.35,
                effective_z_depth: 0.33,
                effective_distance_m: Some(73.0),
                blur_scale: 1.0,
                camera_motion_scale: amigo_2d_spatial::z_depth_to_camera_motion_scale(0.33),
            });

        let plan = build_focus_blur_layer_plan(
            amigo_render_api::FocusBlur2d::default(),
            &[layer("weather.rain.mid", 0.0)],
            Some(&capture_input),
        );

        assert_eq!(plan.z_depth_layers[0].z_depth, 0.33);
        assert_eq!(plan.z_depth_layers[0].base_z_depth, 0.35);
        assert_eq!(plan.z_depth_layers[0].effective_distance_m, Some(73.0));
    }
}
