use super::{BeaconLight2dDrawCommand, BeaconLight2dSceneService};

pub struct Beacon2dRenderExtractionContext<'a> {
    pub beacon_scene_service: &'a BeaconLight2dSceneService,
}

pub trait Beacon2dRenderOutput {
    fn push_beacon2d_render_command(&mut self, command: BeaconLight2dDrawCommand);
}

pub struct Beacon2dRenderExtractor;

impl Beacon2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "beacon_2d"
    }

    pub fn extract(
        &self,
        ctx: Beacon2dRenderExtractionContext<'_>,
        output: &mut impl Beacon2dRenderOutput,
    ) {
        for command in ctx.beacon_scene_service.draw_commands() {
            output.push_beacon2d_render_command(command);
        }
    }
}
