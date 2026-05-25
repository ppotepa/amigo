use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    EventPipelineStepSceneCommand, SceneCommand, SceneEvent, SceneEventQueue, SceneService,
    format_scene_command,
};

use crate::{EventPipeline, EventPipelineService, EventPipelineStep};

pub struct EventPipelineSceneCommandHandler;

pub struct EventPipelineSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub event_pipeline_service: &'a EventPipelineService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct EventPipelineSceneCommandOutcome {
    pub id: String,
    pub source_mod: String,
}

pub fn can_handle_event_pipeline_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::EVENT_PIPELINE_PLUGIN_SCENE_COMMAND_TYPE
    )
}

pub fn handle_event_pipeline_scene_command(
    ctx: EventPipelineSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<EventPipelineSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::EVENT_PIPELINE_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<amigo_scene::EventPipelineSceneCommand>()
                .ok_or_else(|| {
                    AmigoError::Message(
                        "event pipeline plugin scene command payload type mismatch".to_owned(),
                    )
                })?
                .clone();
            handle_event_pipeline_command(ctx, command)
        }
        _ => Err(AmigoError::Message(format!(
            "event-pipeline cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

fn handle_event_pipeline_command(
    ctx: EventPipelineSceneCommandContext<'_>,
    command: amigo_scene::EventPipelineSceneCommand,
) -> AmigoResult<EventPipelineSceneCommandOutcome> {
    let entity = ctx
        .scene_service
        .find_or_spawn_named_entity(command.entity_name.clone());
    ctx.event_pipeline_service.queue(EventPipeline {
        id: command.id.clone(),
        topic: command.topic,
        steps: command
            .steps
            .into_iter()
            .map(event_pipeline_step_from_scene_command)
            .collect(),
    });
    ctx.scene_event_queue.publish(SceneEvent::EventPipelineQueued {
        entity_id: entity.raw(),
        entity_name: command.entity_name,
    });
    Ok(EventPipelineSceneCommandOutcome {
        id: command.id,
        source_mod: command.source_mod,
    })
}

fn event_pipeline_step_from_scene_command(
    step: EventPipelineStepSceneCommand,
) -> EventPipelineStep {
    match step {
        EventPipelineStepSceneCommand::PlayAudio { clip } => EventPipelineStep::PlayAudio { clip },
        EventPipelineStepSceneCommand::SetState { key, value } => {
            EventPipelineStep::SetState { key, value }
        }
        EventPipelineStepSceneCommand::IncrementState { key, by } => {
            EventPipelineStep::IncrementState { key, by }
        }
        EventPipelineStepSceneCommand::ShowUi { path } => EventPipelineStep::ShowUi { path },
        EventPipelineStepSceneCommand::HideUi { path } => EventPipelineStep::HideUi { path },
        EventPipelineStepSceneCommand::BurstParticles { emitter, count } => {
            EventPipelineStep::BurstParticles { emitter, count }
        }
        EventPipelineStepSceneCommand::TransitionScene { scene } => {
            EventPipelineStep::TransitionScene { scene }
        }
        EventPipelineStepSceneCommand::EmitEvent { topic, payload } => {
            EventPipelineStep::EmitEvent { topic, payload }
        }
        EventPipelineStepSceneCommand::Script { function } => {
            EventPipelineStep::Script { function }
        }
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for EventPipelineSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_event_pipeline_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let event_pipeline_service = runtime.required::<EventPipelineService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_event_pipeline_scene_command(
            EventPipelineSceneCommandContext {
                scene_service: scene_service.as_ref(),
                event_pipeline_service: event_pipeline_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
