use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneComposition2dCommandHandler;

impl SceneCommandHandler for SceneComposition2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-composition-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_composition::can_handle_composition_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        let _outcome = amigo_2d_composition::handle_composition_scene_command(
            amigo_2d_composition::CompositionSceneCommandContext {
                render_layer2d_scene_service: ctx.render_layer2d_scene_service,
                light_route2d_scene_service: ctx.light_route2d_scene_service,
            },
            command,
        )?;

        Ok(())
    }
}
