use amigo_scene::SceneService;

use crate::{DepthAuxMap2dDrawCommand, DepthMap2dDrawCommand, DepthMap2dSceneService};

#[derive(Clone, Copy)]
pub struct DepthMap2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub depth_map_scene_service: &'a DepthMap2dSceneService,
}

pub trait DepthMap2dRenderOutput {
    fn push_depth_map2d_render_command(&mut self, command: DepthMap2dDrawCommand);
    fn push_depth_aux_map2d_render_command(&mut self, _command: DepthAuxMap2dDrawCommand) {}
}

pub struct DepthMap2dRenderExtractor;

impl DepthMap2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "depth_map_2d"
    }

    pub fn extract(
        &self,
        ctx: DepthMap2dRenderExtractionContext<'_>,
        output: &mut impl DepthMap2dRenderOutput,
    ) {
        for command in extract_depth_map2d_render_commands(ctx) {
            output.push_depth_map2d_render_command(command);
        }
        for command in extract_depth_aux_map2d_render_commands(ctx) {
            output.push_depth_aux_map2d_render_command(command);
        }
    }
}

pub fn extract_depth_map2d_render_commands(
    ctx: DepthMap2dRenderExtractionContext<'_>,
) -> Vec<DepthMap2dDrawCommand> {
    ctx.depth_map_scene_service
        .commands()
        .into_iter()
        .filter(|command| is_entity_render_visible(ctx.scene_service, &command.entity_name))
        .collect()
}

pub fn extract_depth_aux_map2d_render_commands(
    ctx: DepthMap2dRenderExtractionContext<'_>,
) -> Vec<DepthAuxMap2dDrawCommand> {
    ctx.depth_map_scene_service
        .aux_commands()
        .into_iter()
        .filter(|command| is_entity_render_visible(ctx.scene_service, &command.entity_name))
        .collect()
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}
