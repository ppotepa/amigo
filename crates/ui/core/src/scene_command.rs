use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, UiModelBindingKindSceneCommand,
    format_scene_command,
};

use crate::scene_bridge::convert_scene_ui_style;
use crate::{
    UiDrawCommand, UiModelBinding, UiModelBindingKind, UiModelBindingService, UiSceneService,
    UiStateService, UiTheme, UiThemePalette, UiThemeService, scene_ui_document_to_runtime_document,
};

pub struct UiSceneCommandHandler;

pub struct UiSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub scene_event_queue: &'a SceneEventQueue,
    pub ui_scene_service: &'a UiSceneService,
    pub ui_state_service: &'a UiStateService,
    pub ui_model_binding_service: &'a UiModelBindingService,
    pub ui_theme_service: &'a UiThemeService,
}

pub enum UiSceneCommandOutcome {
    ThemeSet {
        entity_name: String,
        source_mod: String,
        theme_count: usize,
    },
    Document {
        entity_name: String,
        source_mod: String,
    },
    ModelBindings {
        entity_name: String,
        source_mod: String,
    },
}

pub fn can_handle_ui_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::UI_DOCUMENT_PLUGIN_SCENE_COMMAND_TYPE
                || command.command_type == amigo_scene::UI_THEME_SET_PLUGIN_SCENE_COMMAND_TYPE
                || command.command_type == amigo_scene::UI_MODEL_BINDINGS_PLUGIN_SCENE_COMMAND_TYPE
    )
}

pub fn handle_ui_scene_command(
    ctx: UiSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<UiSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::UI_THEME_SET_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<amigo_scene::UiThemeSetSceneCommand>()
                .ok_or_else(|| {
                    AmigoError::Message(
                        "ui theme plugin scene command payload type mismatch".into(),
                    )
                })?
                .clone();
            handle_ui_theme_set_scene_command(ctx, command)
        }
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::UI_DOCUMENT_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<amigo_scene::UiSceneCommand>()
                .ok_or_else(|| {
                    AmigoError::Message(
                        "ui document plugin scene command payload type mismatch".into(),
                    )
                })?
                .clone();
            handle_ui_document_scene_command(ctx, command)
        }
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::UI_MODEL_BINDINGS_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<amigo_scene::UiModelBindingsSceneCommand>()
                .ok_or_else(|| {
                    AmigoError::Message(
                        "ui model bindings plugin scene command payload type mismatch".into(),
                    )
                })?
                .clone();
            handle_ui_model_bindings_scene_command(ctx, command)
        }
        _ => Err(AmigoError::Message(format!(
            "ui scene command handler cannot handle {}",
            format_scene_command(&command)
        ))),
    }
}

fn handle_ui_theme_set_scene_command(
    ctx: UiSceneCommandContext<'_>,
    command: amigo_scene::UiThemeSetSceneCommand,
) -> AmigoResult<UiSceneCommandOutcome> {
    let entity = ctx
        .scene_service
        .find_or_spawn_named_entity(command.entity_name.clone());
    for theme in &command.themes {
        ctx.ui_theme_service
            .register_theme(UiTheme::from_palette_and_classes(
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
                theme
                    .classes
                    .iter()
                    .map(|(name, style)| (name.clone(), convert_scene_ui_style(style)))
                    .collect(),
            ));
    }
    if let Some(active) = command.active.as_deref() {
        let _ = ctx.ui_theme_service.set_active_theme(active);
    }
    ctx.scene_event_queue.publish(SceneEvent::UiThemeSetQueued {
        entity_id: entity.raw(),
        entity_name: command.entity_name.clone(),
    });
    Ok(UiSceneCommandOutcome::ThemeSet {
        entity_name: command.entity_name,
        source_mod: command.source_mod,
        theme_count: command.themes.len(),
    })
}

fn handle_ui_document_scene_command(
    ctx: UiSceneCommandContext<'_>,
    command: amigo_scene::UiSceneCommand,
) -> AmigoResult<UiSceneCommandOutcome> {
    let entity = ctx
        .scene_service
        .find_or_spawn_named_entity(command.entity_name.clone());
    ctx.ui_scene_service.queue(UiDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        document: scene_ui_document_to_runtime_document(&command.document),
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
    Ok(UiSceneCommandOutcome::Document {
        entity_name: command.entity_name,
        source_mod: command.source_mod,
    })
}

fn handle_ui_model_bindings_scene_command(
    ctx: UiSceneCommandContext<'_>,
    command: amigo_scene::UiModelBindingsSceneCommand,
) -> AmigoResult<UiSceneCommandOutcome> {
    let entity = ctx
        .scene_service
        .find_or_spawn_named_entity(command.entity_name.clone());
    for binding in command.bindings {
        ctx.ui_model_binding_service.queue(UiModelBinding {
            path: binding.path,
            state_key: binding.state_key,
            kind: ui_model_binding_kind_from_scene_command(binding.kind),
            format: binding.format,
        });
    }
    ctx.scene_event_queue
        .publish(SceneEvent::UiModelBindingsQueued {
            entity_id: entity.raw(),
            entity_name: command.entity_name.clone(),
        });
    Ok(UiSceneCommandOutcome::ModelBindings {
        entity_name: command.entity_name,
        source_mod: command.source_mod,
    })
}

fn ui_model_binding_kind_from_scene_command(
    kind: UiModelBindingKindSceneCommand,
) -> UiModelBindingKind {
    match kind {
        UiModelBindingKindSceneCommand::Text => UiModelBindingKind::Text,
        UiModelBindingKindSceneCommand::Value => UiModelBindingKind::Value,
        UiModelBindingKindSceneCommand::Height => UiModelBindingKind::Height,
        UiModelBindingKindSceneCommand::Visible => UiModelBindingKind::Visible,
        UiModelBindingKindSceneCommand::Enabled => UiModelBindingKind::Enabled,
        UiModelBindingKindSceneCommand::Selected => UiModelBindingKind::Selected,
        UiModelBindingKindSceneCommand::Options => UiModelBindingKind::Options,
        UiModelBindingKindSceneCommand::Color => UiModelBindingKind::Color,
        UiModelBindingKindSceneCommand::Background => UiModelBindingKind::Background,
        UiModelBindingKindSceneCommand::Theme => UiModelBindingKind::Theme,
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for UiSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_ui_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;
        let ui_scene_service = runtime.required::<UiSceneService>()?;
        let ui_state_service = runtime.required::<UiStateService>()?;
        let ui_model_binding_service = runtime.required::<UiModelBindingService>()?;
        let ui_theme_service = runtime.required::<UiThemeService>()?;

        handle_ui_scene_command(
            UiSceneCommandContext {
                scene_service: scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
                ui_scene_service: ui_scene_service.as_ref(),
                ui_state_service: ui_state_service.as_ref(),
                ui_model_binding_service: ui_model_binding_service.as_ref(),
                ui_theme_service: ui_theme_service.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
