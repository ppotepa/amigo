use amigo_2d_post_fx::{PostFx2d, PostFx2dStack};
use amigo_render_api::{
    DebugOverlayPassPlan, FrameCompositionPlan, PostFxPassPlan, PresentPassPlan,
    RenderPassInput, RenderPassOutput, RenderPassPlan, RenderTargetPlan, UiPassPlan, World2DPassPlan,
};

use super::context::AppRenderFramePacket;

#[derive(Debug, Clone, Default)]
pub(crate) struct AppFrameCompositionBuilder;

impl AppFrameCompositionBuilder {
    pub(crate) fn build(packet: &AppRenderFramePacket) -> FrameCompositionPlan {
        let has_world_2d = packet.has_world_2d();
        let has_world_3d = packet.has_world_3d();
        let post_fx = active_post_fx(packet.post_fx_stack());
        let has_post_fx = !post_fx.is_empty();
        let has_game_ui = !packet.game_ui_overlay().is_empty();
        let has_debug = !packet.debug_overlay().is_empty();

        let mut passes = Vec::new();

        if has_world_2d || has_world_3d {
            passes.push(RenderPassPlan::World2D(World2DPassPlan {
                output: RenderPassOutput::WorldColor,
            }));
        }

        let mut current_input = if has_world_2d || has_world_3d {
            RenderPassInput::WorldColor
        } else {
            RenderPassInput::Surface
        };
        let mut current_output = if has_world_2d || has_world_3d {
            RenderPassOutput::WorldColor
        } else {
            RenderPassOutput::Surface
        };

        for (effect_index, effect) in post_fx.iter().enumerate() {
            let feature_id = amigo_render_api::RenderFeatureId::new(effect.kind());
            let output = if current_output == RenderPassOutput::WorldColor {
                RenderPassOutput::PostFxColor
            } else {
                RenderPassOutput::WorldColor
            };

            passes.push(RenderPassPlan::PostFx(PostFxPassPlan {
                feature_id,
                effect_index,
                input: current_input,
                output,
            }));

            current_input = output.into_input();
            current_output = output;
        }

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
            current_input = current_output.into_input();
        }

        passes.push(RenderPassPlan::Present(PresentPassPlan {
            input: if has_game_ui || has_debug {
                current_input
            } else if has_post_fx {
                current_output.into_input()
            } else {
                if has_world_2d || has_world_3d {
                    RenderPassInput::WorldColor
                } else {
                    RenderPassInput::Surface
                }
            },
        }));

        FrameCompositionPlan::single_main_view(passes)
    }

    pub(crate) fn build_for_target(
        packet: &AppRenderFramePacket,
        target: RenderTargetPlan,
    ) -> FrameCompositionPlan {
        let mut plan = Self::build(packet);
        if let Some(view) = plan.views.first_mut() {
            view.target = target;
        }
        plan
    }
}

fn active_post_fx(stack: Option<&PostFx2dStack>) -> Vec<PostFx2d> {
    stack
        .map(|stack| {
            stack
                .effects
                .iter()
                .copied()
                .filter(PostFx2d::is_active)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
