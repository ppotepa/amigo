//! 2D text scene service for labels and HUD-style content.
//! It queues world-space text state that the renderer consumes for captions and lightweight UI.

use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_math::{Transform2, Vec2};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{SceneEntityId, SceneService, Text2dSceneCommand};
mod runtime_capabilities;
mod render_extraction;
mod scene_command;
mod script_command;
#[cfg(test)]
mod tests;
mod editor_capability;
pub use runtime_capabilities::*;
pub use editor_capability::*;
pub use render_extraction::*;
pub use scene_command::*;
pub use script_command::*;

#[derive(Debug, Clone)]
pub struct Text2d {
    pub content: String,
    pub font: AssetKey,
    pub bounds: Vec2,
    pub transform: Transform2,
}

#[derive(Debug, Clone)]
pub struct Text2dDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub render_layer: String,
    pub text: Text2d,
    pub z_index: f32,
}

#[derive(Debug, Default)]
pub struct Text2dSceneService {
    commands: Mutex<Vec<Text2dDrawCommand>>,
}

impl Text2dSceneService {
    pub fn queue(&self, command: Text2dDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("text2d scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("text2d scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<Text2dDrawCommand> {
        let commands = self
            .commands
            .lock()
            .expect("text2d scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Text2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Text2dPlugin;

impl RuntimePlugin for Text2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-text"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Text2dSceneService::default())?;
        registry.register(Text2dDomainInfo {
            crate_name: "amigo-2d-text",
            capability: "text_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-2d-text",
            &["text_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::Text2dSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::Text2dScriptCommandHandler,
        );
        Ok(())
    }
}

pub fn queue_text2d_scene_command(
    scene_service: &SceneService,
    text_scene_service: &Text2dSceneService,
    command: &Text2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    text_scene_service.queue(Text2dDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        render_layer: command.render_layer.clone(),
        text: Text2d {
            content: command.content.clone(),
            font: command.font.clone(),
            bounds: command.bounds,
            transform: command.transform,
        },
        z_index: command.z_index,
    });
    entity
}

