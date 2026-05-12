use crate::{
    LightRoute2dCommand, LightRoute2dSceneService, RenderLayer2dCommand,
    RenderLayer2dSceneService,
};

pub struct Composition2dRenderExtractionContext<'a> {
    pub render_layer2d_scene_service: &'a RenderLayer2dSceneService,
    pub light_route2d_scene_service: &'a LightRoute2dSceneService,
}

#[derive(Debug, Default, Clone)]
pub struct Composition2dRenderCommands {
    pub render_layers: Vec<RenderLayer2dCommand>,
    pub light_routes: Vec<LightRoute2dCommand>,
}

pub fn extract_composition2d_render_commands(
    ctx: Composition2dRenderExtractionContext<'_>,
) -> Composition2dRenderCommands {
    Composition2dRenderCommands {
        render_layers: ctx.render_layer2d_scene_service.commands(),
        light_routes: ctx.light_route2d_scene_service.commands(),
    }
}
