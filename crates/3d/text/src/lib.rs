use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_math::Transform3;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone)]
pub struct Text3d {
    pub content: String,
    pub font: AssetKey,
    pub size: f32,
    pub transform: Transform3,
}

#[derive(Debug, Clone)]
pub struct Text3dDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub text: Text3d,
}

#[derive(Debug, Default)]
pub struct Text3dSceneService {
    commands: Mutex<Vec<Text3dDrawCommand>>,
}

impl Text3dSceneService {
    pub fn queue(&self, command: Text3dDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("text3d scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("text3d scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<Text3dDrawCommand> {
        let commands = self
            .commands
            .lock()
            .expect("text3d scene service mutex should not be poisoned");
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
pub struct Text3dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Text3dPlugin;

impl RuntimePlugin for Text3dPlugin {
    fn name(&self) -> &'static str {
        "amigo-3d-text"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Text3dSceneService::default())?;
        registry.register(Text3dDomainInfo {
            crate_name: "amigo-3d-text",
            capability: "text_3d",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Text3d, Text3dDrawCommand, Text3dSceneService};
    use amigo_assets::AssetKey;
    use amigo_math::Transform3;
    use amigo_scene::SceneEntityId;

    #[test]
    fn stores_text3d_draw_commands() {
        let service = Text3dSceneService::default();

        service.queue(Text3dDrawCommand {
            entity_id: SceneEntityId::new(21),
            entity_name: "playground-3d-hello".to_owned(),
            text: Text3d {
                content: "HELLO WORLD".to_owned(),
                font: AssetKey::new("playground-3d/fonts/debug-3d"),
                size: 0.5,
                transform: Transform3::default(),
            },
        });

        assert_eq!(service.commands().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["playground-3d-hello".to_owned()]
        );

        service.clear();
        assert!(service.commands().is_empty());
    }
}
