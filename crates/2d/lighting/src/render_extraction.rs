use crate::{
    GlobalLight2dCommand, GlobalLight2dSceneService, LightGroup2dCommand,
    LightGroup2dSceneService, LightMap2dSceneService, LightMap2dSourceCommand,
};

pub struct Lighting2dRenderExtractionContext<'a> {
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub lightmap2d_scene_service: &'a LightMap2dSceneService,
    pub light_group2d_scene_service: &'a LightGroup2dSceneService,
}

#[derive(Debug, Default, Clone)]
pub struct Lighting2dRenderCommands {
    pub global_lights: Vec<GlobalLight2dCommand>,
    pub lightmaps: Vec<LightMap2dSourceCommand>,
    pub light_groups: Vec<LightGroup2dCommand>,
}

pub trait Lighting2dRenderOutput {
    fn push_global_light2d_command(&mut self, command: GlobalLight2dCommand);
    fn push_lightmap2d_command(&mut self, command: LightMap2dSourceCommand);
    fn push_light_group2d_command(&mut self, command: LightGroup2dCommand);
}

pub struct Lighting2dRenderExtractor;

impl Lighting2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "lighting_2d"
    }

    pub fn extract(
        &self,
        ctx: Lighting2dRenderExtractionContext<'_>,
        output: &mut impl Lighting2dRenderOutput,
    ) {
        let commands = extract_lighting2d_render_commands(ctx);
        for command in commands.global_lights {
            output.push_global_light2d_command(command);
        }
        for command in commands.lightmaps {
            output.push_lightmap2d_command(command);
        }
        for command in commands.light_groups {
            output.push_light_group2d_command(command);
        }
    }
}

pub fn extract_lighting2d_render_commands(
    ctx: Lighting2dRenderExtractionContext<'_>,
) -> Lighting2dRenderCommands {
    Lighting2dRenderCommands {
        global_lights: ctx.global_light2d_scene_service.commands(),
        lightmaps: ctx.lightmap2d_scene_service.commands(),
        light_groups: ctx.light_group2d_scene_service.commands(),
    }
}

