use amigo_2d_post_fx::{PostFx2d, PostFx2dStack};
use amigo_render_api::{
    BlendMode, CameraBinding, ClearMode, CompositionLayer, DebugOverlayPassPlan, DepthMode,
    FrameCompositionPlan, PostFxPassPlan, PresentPassPlan, RenderLayerId, RenderPassOutput,
    RenderPassPlan, RenderSpace, RenderTargetPlan, UiPassPlan, WorldPassPlan,
};
use amigo_render_wgpu::WgpuRenderFramePacket;

#[derive(Debug, Clone, Default)]
pub struct WgpuFrameCompositionBuilder;

impl WgpuFrameCompositionBuilder {
    pub fn build(packet: &WgpuRenderFramePacket) -> FrameCompositionPlan {
        let post_fx = active_post_fx(packet.post_fx_stack());
        let has_game_ui = !packet.game_ui_overlay().is_empty();
        let has_debug = !packet.debug_overlay().is_empty();
        let has_frame_content = has_game_ui || has_debug || !post_fx.is_empty();

        let mut passes = vec![RenderPassPlan::World(WorldPassPlan {
            output: RenderPassOutput::WorldColor,
        })];

        let mut current_input = RenderPassOutput::WorldColor.into_input();
        let mut current_output = RenderPassOutput::WorldColor;

        for (effect_index, effect) in post_fx {
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

fn active_post_fx(stack: Option<&PostFx2dStack>) -> Vec<(usize, PostFx2d)> {
    stack
        .map(|stack| {
            stack
                .effects
                .iter()
                .cloned()
                .enumerate()
                .filter(|(_, effect)| effect.is_active())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
