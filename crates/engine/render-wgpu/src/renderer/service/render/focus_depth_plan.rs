use crate::renderer::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct FocusBlurZDepthLayer {
    pub(super) layer_id: String,
    pub(super) order: f32,
    pub(super) z_depth: f32,
    pub(super) blur_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct FocusBlurLayerPlan {
    pub(super) depth_map_layers: BTreeSet<String>,
    pub(super) z_depth_layers: Vec<FocusBlurZDepthLayer>,
    pub(super) overlay_layers: BTreeSet<String>,
    pub(super) legacy_affected_layers: Option<BTreeSet<String>>,
    pub(super) has_explicit_render_depth: bool,
}

pub(super) fn focus_blur_effect_for(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
    host_id: &amigo_2d_post_fx::PostFxHost2dId,
    effect_id: &amigo_2d_post_fx::PostFx2dId,
) -> Option<amigo_2d_post_fx::FocusBlur2d> {
    stacks
        .iter()
        .find(|stack| &stack.host_id == host_id)
        .and_then(|stack| stack.effects.iter().find(|effect| &effect.id == effect_id))
        .and_then(|instance| match &instance.effect {
            amigo_2d_post_fx::PostFx2d::FocusBlur(effect) => Some(effect.clone()),
            _ => None,
        })
}

pub(super) fn shutter_blur_effect_for(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
    host_id: &amigo_2d_post_fx::PostFxHost2dId,
    effect_id: &amigo_2d_post_fx::PostFx2dId,
) -> Option<amigo_2d_post_fx::ShutterBlur2d> {
    stacks
        .iter()
        .find(|stack| &stack.host_id == host_id)
        .and_then(|stack| stack.effects.iter().find(|effect| &effect.id == effect_id))
        .and_then(|instance| match &instance.effect {
            amigo_2d_post_fx::PostFx2d::ShutterBlur(effect) => Some(effect.clone()),
            _ => None,
        })
}

pub(super) fn focus_blur_layer_plan_for_effect(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
    render_layers: &[RenderLayer2dCommand],
    host_id: &amigo_2d_post_fx::PostFxHost2dId,
    effect_id: &amigo_2d_post_fx::PostFx2dId,
) -> Option<FocusBlurLayerPlan> {
    let effect = focus_blur_effect_for(stacks, host_id, effect_id)?;
    Some(build_focus_blur_layer_plan(effect, render_layers))
}

pub(super) fn focus_blur_layer_plan(
    stacks: &[amigo_2d_post_fx::ScopedPostFx2dStack],
    render_layers: &[RenderLayer2dCommand],
) -> Option<FocusBlurLayerPlan> {
    let effect = stacks.iter().find_map(|stack| {
        stack
            .effects
            .iter()
            .find_map(|instance| match &instance.effect {
                amigo_2d_post_fx::PostFx2d::FocusBlur(effect) => Some(effect.clone()),
                _ => None,
            })
    })?;
    Some(build_focus_blur_layer_plan(effect, render_layers))
}

pub(super) fn build_focus_blur_layer_plan(
    effect: amigo_2d_post_fx::FocusBlur2d,
    render_layers: &[RenderLayer2dCommand],
) -> FocusBlurLayerPlan {
    let has_explicit_render_depth = render_layers
        .iter()
        .any(|layer| !layer.depth.is_depth_map());
    let legacy_affected_layers = (!has_explicit_render_depth && !effect.affected_layers.is_empty())
        .then(|| effect.affected_layers.into_iter().collect::<BTreeSet<_>>());

    let mut depth_map_layers = BTreeSet::new();
    let mut z_depth_layers = Vec::new();
    let mut overlay_layers = BTreeSet::new();

    for layer in render_layers {
        match layer.depth.mode {
            amigo_2d_composition::RenderDepthMode2d::DepthMap => {
                depth_map_layers.insert(layer.id.clone());
            }
            amigo_2d_composition::RenderDepthMode2d::Distance
            | amigo_2d_composition::RenderDepthMode2d::ZDepth => {
                z_depth_layers.push(FocusBlurZDepthLayer {
                    layer_id: layer.id.clone(),
                    order: layer.order,
                    z_depth: layer.depth.z_depth,
                    blur_scale: layer.depth.blur_scale,
                });
            }
            amigo_2d_composition::RenderDepthMode2d::Infinity => {
                z_depth_layers.push(FocusBlurZDepthLayer {
                    layer_id: layer.id.clone(),
                    order: layer.order,
                    z_depth: 0.0,
                    blur_scale: layer.depth.blur_scale,
                });
            }
            amigo_2d_composition::RenderDepthMode2d::Overlay => {
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
        legacy_affected_layers,
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
            depth: amigo_2d_composition::RenderDepth2d {
                mode: amigo_2d_composition::RenderDepthMode2d::Distance,
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
            amigo_2d_post_fx::FocusBlur2d::default(),
            &[
                layer("weather.rain.super_near", 60.0),
                layer("title.depth2d", 20.0),
                layer("weather.rain.1m", 35.0),
            ],
        );

        let layer_ids = plan
            .z_depth_layers
            .iter()
            .map(|layer| layer.layer_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            layer_ids,
            vec!["title.depth2d", "weather.rain.1m", "weather.rain.super_near"]
        );
    }
}
