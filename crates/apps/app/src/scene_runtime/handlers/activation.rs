use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;
use amigo_scene::{ActivationEntrySceneCommand, ActivationSetSceneCommand};

pub(crate) struct SceneActivationCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneActivationCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-activation"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_scene::can_handle_scene_activation_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_scene::handle_scene_activation_scene_command(
            amigo_scene::SceneActivationCommandContext {
                activation_set_scene_service: ctx.activation_set_scene_service,
            },
            command,
        )?;

        match outcome {
            amigo_scene::SceneActivationCommandOutcome::Queued {
                id,
                source_mod,
                entry_count,
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued activation set `{}` with {} entries from mod `{}`",
                    id, entry_count, source_mod
                ));
                Ok(())
            }
            amigo_scene::SceneActivationCommandOutcome::Activate { set } => {
                let Some(set) = set else {
                    ctx.dev_console_state
                        .write_line("unknown activation set".to_string());
                    return Ok(());
                };
                let applied = apply_activation_set(ctx, &set);
                ctx.dev_console_state.write_line(format!(
                    "activated set `{}` across {applied} entities",
                    set.id
                ));
                Ok(())
            }
        }
    }
}

fn apply_activation_set(
    ctx: &AppSceneCommandContext<'_>,
    set: &ActivationSetSceneCommand,
) -> usize {
    let mut applied = 0;
    for entry in &set.entries {
        for entity_name in resolve_activation_targets(ctx, &entry.target) {
            apply_activation_entry(ctx, &entity_name, entry);
            applied += 1;
        }
    }
    applied
}

fn resolve_activation_targets(
    ctx: &AppSceneCommandContext<'_>,
    selector: &amigo_scene::EntitySelector,
) -> Vec<String> {
    amigo_scene::resolve_activation_targets(
        amigo_scene::ActivationTargetResolveContext {
            scene_service: ctx.scene_service,
            entity_pool_scene_service: ctx.entity_pool_scene_service,
        },
        selector,
    )
}

fn apply_activation_entry(
    ctx: &AppSceneCommandContext<'_>,
    entity_name: &str,
    entry: &ActivationEntrySceneCommand,
) {
    if let Some(mut lifecycle) = ctx.scene_service.lifecycle_of(entity_name) {
        if let Some(visible) = entry.lifecycle.visible {
            lifecycle.visible = visible;
        }
        if let Some(enabled) = entry.lifecycle.simulation_enabled {
            lifecycle.simulation_enabled = enabled;
        }
        if let Some(enabled) = entry.lifecycle.collision_enabled {
            lifecycle.collision_enabled = enabled;
        }
        let _ = ctx.scene_service.set_lifecycle(entity_name, lifecycle);
    }

    if let Some(transform) = entry.transform {
        let _ = ctx.scene_service.set_transform(entity_name, transform);
    }

    if !entry.properties.is_empty() {
        if let Some(entity) = ctx.scene_service.entity_by_name(entity_name) {
            let mut properties = entity.properties;
            properties.extend(entry.properties.clone());
            let _ = ctx.scene_service.configure_entity_metadata(
                entity_name,
                entity.lifecycle,
                entity.tags,
                entity.groups,
                properties,
            );
        }
    }

    if let Some(velocity) = entry.velocity {
        let _ = ctx.motion_scene_service.set_velocity(entity_name, velocity);
        if let Some(mut state) = ctx.motion_scene_service.freeflight_state(entity_name) {
            state.velocity = velocity;
            let _ = ctx
                .motion_scene_service
                .sync_freeflight_state(entity_name, state);
        }
        if let Some(mut body_state) = ctx.physics_scene_service.body_state(entity_name) {
            body_state.velocity = velocity;
            let _ = ctx
                .physics_scene_service
                .sync_body_state(entity_name, body_state);
        }
    }
}


