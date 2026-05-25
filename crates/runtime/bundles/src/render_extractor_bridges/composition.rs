use amigo_composite_plugin::{
    PostFx2d, PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d, ScopedPostFx2dStack,
};
use amigo_render_api::{
    BlendMode, CameraBinding, ClearMode, CompositionLayer, DebugOverlayPassPlan, DepthMode,
    FrameCompositionPlan, PostFxPassPlan, PresentPassPlan, RenderLayerId, RenderPassOutput,
    RenderPassPlan, RenderSpace, RenderTargetPlan, UiPassPlan, WorldPassPlan,
};
use amigo_render_wgpu::WgpuRenderFramePacket;

#[derive(Debug, Clone, Default)]
pub struct WgpuFrameCompositionBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WgpuFrameCompositionOptions {
    pub debug_overlay_after_present: bool,
}

impl WgpuFrameCompositionBuilder {
    pub fn build(packet: &WgpuRenderFramePacket) -> FrameCompositionPlan {
        Self::build_with_options(packet, WgpuFrameCompositionOptions::default())
    }

    pub fn build_with_options(
        packet: &WgpuRenderFramePacket,
        options: WgpuFrameCompositionOptions,
    ) -> FrameCompositionPlan {
        let post_fx = active_post_fx(packet.post_fx_stacks());
        let has_game_ui = !packet.game_ui_overlay().is_empty();
        let has_debug = !packet.debug_overlay().is_empty() && !options.debug_overlay_after_present;
        let has_frame_content = has_game_ui || has_debug || !post_fx.is_empty();

        let mut passes = vec![RenderPassPlan::World(WorldPassPlan {
            output: RenderPassOutput::WorldColor,
        })];

        let mut current_input = RenderPassOutput::WorldColor.into_input();
        let mut current_output = RenderPassOutput::WorldColor;

        append_post_fx_passes(
            &mut passes,
            post_fx,
            &mut current_input,
            &mut current_output,
        );

        if has_game_ui {
            passes.push(RenderPassPlan::GameUi(UiPassPlan {
                input: current_output.into_input(),
                output: current_output,
            }));
            current_input = current_output.into_input();
        }

        if has_debug {
            passes.push(RenderPassPlan::DebugOverlay(DebugOverlayPassPlan {
                input: current_output.into_input(),
                output: current_output,
            }));
        }

        let present_input = if has_frame_content {
            current_input
        } else {
            RenderPassOutput::WorldColor.into_input()
        };

        passes.push(RenderPassPlan::Present(PresentPassPlan {
            input: present_input,
        }));

        FrameCompositionPlan::single_main_view(passes)
            .with_layers(wgpu_composition_layers(RenderTargetPlan::Surface))
    }

    pub fn build_for_target(
        packet: &WgpuRenderFramePacket,
        target: RenderTargetPlan,
    ) -> FrameCompositionPlan {
        let mut plan = Self::build(packet);
        if let Some(view) = plan.views.first_mut() {
            view.target = target;
        }
        plan = plan.with_layers(wgpu_composition_layers(target));
        plan
    }
}

fn append_post_fx_passes(
    passes: &mut Vec<RenderPassPlan>,
    post_fx: Vec<ActivePostFxPass>,
    current_input: &mut amigo_render_api::RenderPassInput,
    current_output: &mut RenderPassOutput,
) {
    for pass in post_fx {
        let effect = pass.effect.clone();
        let feature_id = amigo_render_api::RenderFeatureId::new(effect.kind());
        let output = if *current_output == RenderPassOutput::WorldColor {
            RenderPassOutput::PostFxColor
        } else {
            RenderPassOutput::WorldColor
        };

        passes.push(RenderPassPlan::PostFx(PostFxPassPlan {
            host_id: pass.host_id,
            effect_id: pass.effect_id,
            scope: pass.scope,
            pipeline: pass.pipeline,
            feature_id,
            input: *current_input,
            output,
        }));

        *current_input = output.into_input();
        *current_output = output;
    }
}

fn wgpu_composition_layers(target: RenderTargetPlan) -> Vec<CompositionLayer> {
    vec![
        CompositionLayer {
            id: RenderLayerId::new("world_3d"),
            space: RenderSpace::World3D,
            camera: Some(CameraBinding::main()),
            order: 0,
            target,
            clear: ClearMode::ClearColor,
            depth: DepthMode::ReadWrite,
            blend: BlendMode::Opaque,
        },
        CompositionLayer {
            id: RenderLayerId::new("world_2d"),
            space: RenderSpace::World2D,
            camera: Some(CameraBinding::main()),
            order: 10,
            target,
            clear: ClearMode::Preserve,
            depth: DepthMode::None,
            blend: BlendMode::Alpha,
        },
        CompositionLayer {
            id: RenderLayerId::new("ui"),
            space: RenderSpace::Ui,
            camera: None,
            order: 100,
            target,
            clear: ClearMode::Preserve,
            depth: DepthMode::None,
            blend: BlendMode::Alpha,
        },
        CompositionLayer {
            id: RenderLayerId::new("debug_overlay"),
            space: RenderSpace::DebugOverlay,
            camera: None,
            order: 200,
            target,
            clear: ClearMode::Preserve,
            depth: DepthMode::None,
            blend: BlendMode::Alpha,
        },
    ]
}

#[derive(Debug, Clone)]
struct ActivePostFxPass {
    host_id: PostFxHost2dId,
    effect_id: PostFx2dId,
    scope: PostFxScope2d,
    pipeline: PostFxPipelineKind,
    effect: PostFx2d,
}

fn active_post_fx(stacks: &[ScopedPostFx2dStack]) -> Vec<ActivePostFxPass> {
    // Frame/FrameGraph, DrawLayer/OffscreenDrawLayer, SceneObjectPixels/OffscreenObject,
    // GroupSubtree/OffscreenGroup, and CachedImage-backed SourceImage/ImagePart scopes currently
    // have render execution.
    stacks
        .iter()
        .filter(|stack| {
            matches!(
                (&stack.scope, stack.pipeline),
                (PostFxScope2d::Frame, PostFxPipelineKind::FrameGraph)
                    | (
                        PostFxScope2d::DrawLayer { .. },
                        PostFxPipelineKind::OffscreenDrawLayer
                    )
                    | (
                        PostFxScope2d::SceneObjectPixels { .. },
                        PostFxPipelineKind::OffscreenObject
                    )
                    | (
                        PostFxScope2d::GroupSubtree { .. },
                        PostFxPipelineKind::OffscreenGroup
                    )
                    | (
                        PostFxScope2d::SourceImage { .. },
                        PostFxPipelineKind::CachedImage
                    )
                    | (
                        PostFxScope2d::ImagePart { .. },
                        PostFxPipelineKind::CachedImage
                    )
            )
        })
        .flat_map(|stack| {
            stack.effects.iter().filter_map(|instance| {
                if !instance.effect.is_active() {
                    return None;
                }
                Some(ActivePostFxPass {
                    host_id: stack.host_id.clone(),
                    effect_id: instance.id.clone(),
                    scope: stack.scope.clone(),
                    pipeline: stack.pipeline,
                    effect: instance.effect.clone(),
                })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_composite_plugin::{
        PostFx2dInstance, PostFxBlur2d, PostFxPipelineKind, PostFxScope2d, ScopedPostFx2dStack,
    };

    fn effect_item() -> PostFx2dInstance {
        PostFx2dInstance::new(
            "blur",
            amigo_render_api::post_fx_blur(PostFxBlur2d::default()),
        )
    }

    #[test]
    fn active_post_fx_includes_draw_layer_offscreen_pipeline() {
        let passes = active_post_fx(&[ScopedPostFx2dStack {
            host_id: amigo_composite_plugin::PostFxHost2dId::new("layer.weather"),
            scope: PostFxScope2d::DrawLayer {
                draw_layer_id: "weather.rain.mid".to_owned(),
            },
            pipeline: PostFxPipelineKind::OffscreenDrawLayer,
            effects: vec![effect_item()],
        }]);

        assert_eq!(passes.len(), 1);
        assert!(matches!(passes[0].scope, PostFxScope2d::DrawLayer { .. }));
        assert_eq!(passes[0].pipeline, PostFxPipelineKind::OffscreenDrawLayer);
    }

    #[test]
    fn active_post_fx_includes_supported_non_frame_scopes() {
        let passes = active_post_fx(&[
            ScopedPostFx2dStack {
                host_id: amigo_composite_plugin::PostFxHost2dId::new("object.rain"),
                scope: PostFxScope2d::SceneObjectPixels {
                    scene_object_id: "rain.mid".to_owned(),
                },
                pipeline: PostFxPipelineKind::OffscreenObject,
                effects: vec![effect_item()],
            },
            ScopedPostFx2dStack {
                host_id: amigo_composite_plugin::PostFxHost2dId::new("source.cached"),
                scope: PostFxScope2d::SourceImage {
                    asset: "test/image".to_owned(),
                },
                pipeline: PostFxPipelineKind::CachedImage,
                effects: vec![effect_item()],
            },
        ]);

        assert_eq!(passes.len(), 2);
        assert!(matches!(
            passes[0].scope,
            PostFxScope2d::SceneObjectPixels { .. }
        ));
        assert_eq!(passes[0].pipeline, PostFxPipelineKind::OffscreenObject);
        assert!(matches!(passes[1].scope, PostFxScope2d::SourceImage { .. }));
        assert_eq!(passes[1].pipeline, PostFxPipelineKind::CachedImage);
    }

    #[test]
    fn active_post_fx_includes_frame_pipeline() {
        let passes = active_post_fx(&[ScopedPostFx2dStack {
            host_id: amigo_composite_plugin::PostFxHost2dId::new("frame"),
            scope: PostFxScope2d::Frame,
            pipeline: PostFxPipelineKind::FrameGraph,
            effects: vec![effect_item()],
        }]);

        assert_eq!(passes.len(), 1);
        assert!(matches!(passes[0].scope, PostFxScope2d::Frame));
    }
}
