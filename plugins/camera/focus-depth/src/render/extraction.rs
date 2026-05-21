use amigo_render_api::{
    RenderContribution2d, RenderDepthAuxMap2d, RenderDepthAuxMap2dChannels, RenderDepthMap2d,
    RenderDepthMapViewportFit2d, RenderExtractionOutput2d,
};
use amigo_scene::SceneService;

use crate::{DepthAuxMap2dDrawCommand, DepthMap2dDrawCommand, DepthMap2dSceneService};

use super::DEPTH_MAP_2D_EXTRACTOR_ID;

#[derive(Clone, Copy)]
pub struct DepthMap2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub depth_map_scene_service: &'a DepthMap2dSceneService,
}

pub struct DepthMap2dRenderExtractor;

impl DepthMap2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        DEPTH_MAP_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: DepthMap2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in extract_depth_map2d_render_commands(ctx) {
            output.push_render_contribution_2d(depth_map_command_to_render_contribution(&command));
        }
        for command in extract_depth_aux_map2d_render_commands(ctx) {
            output.push_render_contribution_2d(depth_aux_map_command_to_render_contribution(
                &command,
            ));
        }
    }
}

pub fn depth_map_command_to_render_contribution(
    command: &DepthMap2dDrawCommand,
) -> RenderContribution2d {
    RenderContribution2d::depth_map_2d(RenderDepthMap2d {
        owner_entity: command.entity_name.clone(),
        id: command.depth_map.id.clone(),
        asset: command.depth_map.asset.clone(),
        size: command.depth_map.size,
        viewport_fit: depth_map_viewport_fit(command.depth_map.viewport_fit),
        white_is_near: command.depth_map.white_is_near,
        z_index: command.z_index,
        transform: command.transform,
    })
}

pub fn depth_aux_map_command_to_render_contribution(
    command: &DepthAuxMap2dDrawCommand,
) -> RenderContribution2d {
    RenderContribution2d::depth_aux_map_2d(RenderDepthAuxMap2d {
        owner_entity: command.entity_name.clone(),
        id: command.depth_aux_map.id.clone(),
        asset: command.depth_aux_map.asset.clone(),
        surface_asset: command.depth_aux_map.surface_asset.clone(),
        size: command.depth_aux_map.size,
        viewport_fit: depth_map_viewport_fit(command.depth_aux_map.viewport_fit),
        channels: RenderDepthAuxMap2dChannels {
            r: command.depth_aux_map.channels.r.clone(),
            g: command.depth_aux_map.channels.g.clone(),
            b: command.depth_aux_map.channels.b.clone(),
            a: command.depth_aux_map.channels.a.clone(),
        },
        z_index: command.z_index,
        transform: command.transform,
    })
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

fn depth_map_viewport_fit(value: crate::DepthMapViewportFit2d) -> RenderDepthMapViewportFit2d {
    match value {
        crate::DepthMapViewportFit2d::Fixed => RenderDepthMapViewportFit2d::Fixed,
        crate::DepthMapViewportFit2d::Stretch => RenderDepthMapViewportFit2d::Stretch,
        crate::DepthMapViewportFit2d::Contain => RenderDepthMapViewportFit2d::Contain,
        crate::DepthMapViewportFit2d::Cover => RenderDepthMapViewportFit2d::Cover,
    }
}
