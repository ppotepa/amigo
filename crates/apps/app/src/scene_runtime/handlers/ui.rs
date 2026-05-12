use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;
use super::super::ui_support;

pub(crate) struct SceneUiCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneUiCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-ui"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_ui::can_handle_ui_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        if let SceneCommand::QueueUi { command } = &command {
            ui_support::register_ui_font_asset_references(
                ctx.asset_catalog,
                &command.source_mod,
                &command.document,
            );
        }

        let outcome = amigo_ui::handle_ui_scene_command(
            amigo_ui::UiSceneCommandContext {
                scene_service: ctx.scene_service,
                scene_event_queue: ctx.scene_event_queue,
                ui_scene_service: ctx.ui_scene_service,
                ui_state_service: ctx.ui_state_service,
                ui_model_binding_service: ctx.ui_model_binding_service,
                ui_theme_service: ctx.ui_theme_service,
            },
            command,
        )?;

        match outcome {
            amigo_ui::UiSceneCommandOutcome::ThemeSet {
                entity_name,
                source_mod,
                theme_count,
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued ui theme set `{}` with {} themes from mod `{}`",
                    entity_name, theme_count, source_mod
                ));
                Ok(())
            }
            amigo_ui::UiSceneCommandOutcome::Document {
                entity_name,
                source_mod,
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued ui document entity `{}` from mod `{}`",
                    entity_name, source_mod
                ));
                Ok(())
            }
            amigo_ui::UiSceneCommandOutcome::ModelBindings {
                entity_name,
                source_mod,
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued ui model bindings `{}` from mod `{}`",
                    entity_name, source_mod
                ));
                Ok(())
            }
        }
    }
}


