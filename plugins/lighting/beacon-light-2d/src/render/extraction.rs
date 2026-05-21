use amigo_render_api::RenderExtractionOutput2d;

use crate::BeaconLight2dSceneService;

use super::BEACON_2D_EXTRACTOR_ID;

pub struct Beacon2dRenderExtractionContext<'a> {
    pub beacon_scene_service: &'a BeaconLight2dSceneService,
}

pub struct Beacon2dRenderExtractor;

impl Beacon2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        BEACON_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: Beacon2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in ctx.beacon_scene_service.draw_commands() {
            output.push_renderable_2d(super::beacon_draw_command_to_renderable_2d(&command));
            output.push_render_contribution_2d(
                super::beacon_draw_command_to_light_contribution(&command),
            );
        }
    }
}
