use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::ui_support;

pub(crate) struct SceneUiCommandHandler;

impl SceneCommandHandler for SceneUiCommandHandler {
    fn name(&self) -> &'static str {
        "scene-ui"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::QueueUi { .. } | SceneCommand::QueueUiThemeSet { .. }
        )
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueUiThemeSet { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                for theme in &command.themes {
                    ctx.ui_theme_service.register_theme(UiTheme::from_palette(
                        theme.id.clone(),
                        UiThemePalette {
                            background: theme.palette.background,
                            surface: theme.palette.surface,
                            surface_alt: theme.palette.surface_alt,
                            text: theme.palette.text,
                            text_muted: theme.palette.text_muted,
                            border: theme.palette.border,
                            accent: theme.palette.accent,
                            accent_text: theme.palette.accent_text,
                            danger: theme.palette.danger,
                            warning: theme.palette.warning,
                            success: theme.palette.success,
                        },
                    ));
                }
                if let Some(active) = command.active.as_deref() {
                    let _ = ctx.ui_theme_service.set_active_theme(active);
                }
                ctx.scene_event_queue.publish(SceneEvent::UiThemeSetQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued ui theme set `{}` with {} themes from mod `{}`",
                    command.entity_name,
                    command.themes.len(),
                    command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueUi { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ui_support::register_ui_font_asset_references(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.document,
                );
                ctx.ui_scene_service.queue(UiDrawCommand {
                    entity_id: entity,
                    entity_name: command.entity_name.clone(),
                    document: ui_support::convert_scene_ui_document(&command.document),
                });
                let root_segment = command
                    .document
                    .root
                    .id
                    .clone()
                    .unwrap_or_else(|| "root".to_owned());
                let root_path = format!("{}.{}", command.entity_name, root_segment);
                if ctx.scene_service.is_visible(&command.entity_name) {
                    let _ = ctx.ui_state_service.show(root_path);
                } else {
                    let _ = ctx.ui_state_service.hide(root_path);
                }
                ctx.scene_event_queue.publish(SceneEvent::UiQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued ui document entity `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}
