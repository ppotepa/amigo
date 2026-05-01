use amigo_event_pipeline::{EventPipeline, EventPipelineStep};
use amigo_scene::{EventPipelineStepSceneCommand, SceneCommand};

use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneEventPipelineCommandHandler;

impl SceneCommandHandler for SceneEventPipelineCommandHandler {
    fn name(&self) -> &'static str {
        "scene-event-pipeline"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueEventPipeline { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueEventPipeline { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.event_pipeline_service.queue(EventPipeline {
                    id: command.id.clone(),
                    topic: command.topic.clone(),
                    steps: command
                        .steps
                        .into_iter()
                        .map(event_pipeline_step_from_scene_command)
                        .collect(),
                });
                ctx.scene_event_queue
                    .publish(SceneEvent::EventPipelineQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued event pipeline `{}` from mod `{}`",
                    command.id, command.source_mod
                ));
                Ok(())
            }
            other => Err(AmigoError::Message(format!(
                "{} cannot handle {}",
                self.name(),
                amigo_scene::format_scene_command(&other)
            ))),
        }
    }
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
    }
}
