use crate::{
    LightRoute2dCommand, LightRoute2dSceneService, RenderLayer2dCommand, RenderLayer2dSceneService,
};

pub const COMPOSITION_2D_EXTRACTOR_ID: &str = "composition_2d";

pub struct Composition2dRenderExtractionContext<'a> {
    pub render_layer2d_scene_service: &'a RenderLayer2dSceneService,
    pub light_route2d_scene_service: &'a LightRoute2dSceneService,
}

#[derive(Debug, Default, Clone)]
pub struct Composition2dRenderCommands {
    pub render_layers: Vec<RenderLayer2dCommand>,
    pub light_routes: Vec<LightRoute2dCommand>,
}

pub trait Composition2dRenderOutput {
    fn push_render_layer2d_command(&mut self, command: RenderLayer2dCommand);
    fn push_light_route2d_command(&mut self, command: LightRoute2dCommand);
}

pub struct Composition2dRenderExtractor;

impl Composition2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        COMPOSITION_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: Composition2dRenderExtractionContext<'_>,
        output: &mut impl Composition2dRenderOutput,
    ) {
        let commands = extract_composition2d_render_commands(ctx);
        for command in commands.render_layers {
            output.push_render_layer2d_command(command);
        }
        for command in commands.light_routes {
            output.push_light_route2d_command(command);
        }
    }
}

pub fn extract_composition2d_render_commands(
    ctx: Composition2dRenderExtractionContext<'_>,
) -> Composition2dRenderCommands {
    Composition2dRenderCommands {
        render_layers: ctx.render_layer2d_scene_service.commands(),
        light_routes: ctx.light_route2d_scene_service.commands(),
    }
}
