use amigo_2d_post_fx::{PostFx2d, PostFx2dStack};
use amigo_render_api::{
    DebugOverlayPassPlan, FrameCompositionPlan, PostFxPassKind, PostFxPassPlan, PresentPassPlan,
    RenderPassInput, RenderPassOutput, RenderPassPlan, UiPassPlan, World2DPassPlan,
    World3DPassPlan,
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

        if has_world_2d {
            passes.push(RenderPassPlan::World2D(World2DPassPlan {
                output: if has_post_fx {
                    RenderPassOutput::WorldColor
                } else {
                    RenderPassOutput::Surface
                },
            }));
        }

        if has_world_3d {
            passes.push(RenderPassPlan::World3D(World3DPassPlan {
                output: if has_post_fx {
                    RenderPassOutput::WorldColor
                } else {
                    RenderPassOutput::Surface
                },
            }));
        }

        let mut current_input = if has_post_fx {
            RenderPassInput::WorldColor
        } else {
            RenderPassInput::Surface
        };

        for effect in post_fx {
            let kind = match effect {
                PostFx2d::LensDroplets(_) => PostFxPassKind::LensDroplets,
                PostFx2d::Blur(_) => PostFxPassKind::Blur,
                PostFx2d::EmbossEdges(_) => PostFxPassKind::EmbossEdges,
            };

            passes.push(RenderPassPlan::PostFx(PostFxPassPlan {
                kind,
                input: current_input,
                output: RenderPassOutput::Surface,
            }));

            current_input = RenderPassInput::Surface;
        }

        if has_game_ui {
            passes.push(RenderPassPlan::GameUi(UiPassPlan {
                input: RenderPassInput::Surface,
                output: RenderPassOutput::Surface,
            }));
        }

        if has_debug {
            passes.push(RenderPassPlan::DebugOverlay(DebugOverlayPassPlan {
                input: RenderPassInput::Surface,
                output: RenderPassOutput::Surface,
            }));
        }

        passes.push(RenderPassPlan::Present(PresentPassPlan {
            input: RenderPassInput::Surface,
        }));

        FrameCompositionPlan::single_main_view(passes)
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
