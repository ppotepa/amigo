use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_math::ColorRgba;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone)]
pub struct Material3d {
    pub label: String,
    pub albedo: ColorRgba,
    pub source: Option<AssetKey>,
}

#[derive(Debug, Clone)]
pub struct MaterialDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub material: Material3d,
}

#[derive(Debug, Default)]
pub struct MaterialSceneService {
    commands: Mutex<Vec<MaterialDrawCommand>>,
}

impl MaterialSceneService {
    pub fn queue(&self, command: MaterialDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("material scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("material scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<MaterialDrawCommand> {
        let commands = self
            .commands
            .lock()
            .expect("material scene service mutex should not be poisoned");
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
pub struct MaterialDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct MaterialPlugin;

impl RuntimePlugin for MaterialPlugin {
    fn name(&self) -> &'static str {
        "amigo-3d-material"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(MaterialSceneService::default())?;
        registry.register(MaterialDomainInfo {
            crate_name: "amigo-3d-material",
            capability: "materials_3d",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Material3d, MaterialDrawCommand, MaterialSceneService};
    use amigo_assets::AssetKey;
    use amigo_math::ColorRgba;
    use amigo_scene::SceneEntityId;

    #[test]
    fn stores_material_draw_commands() {
        let service = MaterialSceneService::default();

        service.queue(MaterialDrawCommand {
            entity_id: SceneEntityId::new(13),
            entity_name: "playground-3d-probe".to_owned(),
            material: Material3d {
                label: "debug-surface".to_owned(),
                albedo: ColorRgba::WHITE,
                source: Some(AssetKey::new("playground-3d/materials/debug-surface")),
            },
        });

        assert_eq!(service.commands().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["playground-3d-probe".to_owned()]
        );

        service.clear();
        assert!(service.commands().is_empty());
    }
}
