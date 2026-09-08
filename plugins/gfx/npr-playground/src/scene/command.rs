use std::{any::Any, sync::Arc};

use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{PluginSceneCommand, PluginSceneCommandPayload, RuntimeSceneCommandHandler, SceneCommand};

use crate::state::NprPlaygroundState;

use super::NprPlaygroundSceneDocument;

pub const NPR_PLAYGROUND_SCENE_COMMAND_TYPE: &str =
    "amigo.gfx.npr-playground.scene-command.NprSettings";

#[derive(Debug, Clone, PartialEq)]
pub struct NprPlaygroundSceneCommand {
    pub settings: NprPlaygroundSceneDocument,
}

#[derive(Debug, Clone, PartialEq)]
struct NprPlaygroundPluginSceneCommandPayload(NprPlaygroundSceneCommand);

impl PluginSceneCommandPayload for NprPlaygroundPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str { NPR_PLAYGROUND_SCENE_COMMAND_TYPE }
    fn command_as_any(&self) -> &dyn Any { &self.0 }
    fn eq_payload(&self, other: &dyn PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<NprPlaygroundSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn npr_playground_plugin_scene_command(command: NprPlaygroundSceneCommand) -> PluginSceneCommand {
    PluginSceneCommand::new(Arc::new(NprPlaygroundPluginSceneCommandPayload(command)))
}

pub struct NprPlaygroundSceneCommandHandler;

impl RuntimeSceneCommandHandler for NprPlaygroundSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::Plugin { command } if command.command_type == NPR_PLAYGROUND_SCENE_COMMAND_TYPE)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let SceneCommand::Plugin { command } = command else {
            return Err(AmigoError::Message("NPR handler received a non-plugin command".to_owned()));
        };
        let settings = command
            .payload_as::<NprPlaygroundSceneCommand>()
            .ok_or_else(|| AmigoError::Message("NPR scene command payload type mismatch".to_owned()))?
            .settings
            .clone();
        runtime.required::<NprPlaygroundState>()?.apply_authored_scene(settings)
            .map_err(AmigoError::Message)
    }
}
