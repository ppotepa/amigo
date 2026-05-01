use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{SceneEntityId, SceneService, Text2dSceneCommand};

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
    pub text: Text2d,
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
        )
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
        text: Text2d {
            content: command.content.clone(),
            font: command.font.clone(),
            bounds: command.bounds,
            transform: command.transform,
        },
    });
    entity
}

#[cfg(test)]
mod tests {
    use super::{Text2d, Text2dDrawCommand, Text2dSceneService, queue_text2d_scene_command};
    use amigo_assets::AssetKey;
    use amigo_math::{Transform2, Vec2};
    use amigo_scene::{SceneEntityId, SceneService, Text2dSceneCommand};

    #[test]
    fn stores_text_draw_commands() {
        let service = Text2dSceneService::default();

        service.queue(Text2dDrawCommand {
            entity_id: SceneEntityId::new(9),
            entity_name: "playground-2d-label".to_owned(),
            text: Text2d {
                content: "AMIGO 2D".to_owned(),
                font: AssetKey::new("playground-2d/fonts/debug-ui"),
                bounds: Vec2::new(320.0, 64.0),
                transform: Transform2::default(),
            },
        });

        assert_eq!(service.commands().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["playground-2d-label".to_owned()]
        );

        service.clear();
        assert!(service.commands().is_empty());
    }

    #[test]
    fn queues_text2d_scene_command() {
        let scene = SceneService::default();
        let service = Text2dSceneService::default();

        let entity = queue_text2d_scene_command(
            &scene,
            &service,
            &Text2dSceneCommand::new(
                "playground-2d",
                "playground-2d-label",
                "AMIGO 2D",
                AssetKey::new("playground-2d/fonts/debug-ui"),
                Vec2::new(320.0, 64.0),
            ),
        );

        assert_eq!(entity.raw(), 0);
        assert_eq!(service.commands().len(), 1);
        assert_eq!(scene.entity_names(), vec!["playground-2d-label".to_owned()]);
    }
}
