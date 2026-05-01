use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_math::Transform3;
use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{Mesh3dSceneCommand, SceneEntityId, SceneService};

#[derive(Debug, Clone)]
pub struct Mesh3d {
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
}

#[derive(Debug, Clone)]
pub struct MeshDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub mesh: Mesh3d,
}

#[derive(Debug, Default)]
pub struct MeshSceneService {
    commands: Mutex<Vec<MeshDrawCommand>>,
}

impl MeshSceneService {
    pub fn queue(&self, command: MeshDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<MeshDrawCommand> {
        let commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
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
pub struct MeshDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct MeshPlugin;

impl RuntimePlugin for MeshPlugin {
    fn name(&self) -> &'static str {
        "amigo-3d-mesh"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(MeshSceneService::default())?;
        registry.register(MeshDomainInfo {
            crate_name: "amigo-3d-mesh",
            capability: "rendering_3d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-3d-mesh",
            &["rendering_3d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}

pub fn queue_mesh_scene_command(
    scene_service: &SceneService,
    mesh_scene_service: &MeshSceneService,
    command: &Mesh3dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    mesh_scene_service.queue(MeshDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        mesh: Mesh3d {
            mesh_asset: command.mesh_asset.clone(),
            transform: command.transform,
        },
    });
    entity
}

#[cfg(test)]
mod tests {
    use super::{Mesh3d, MeshDrawCommand, MeshSceneService, queue_mesh_scene_command};
    use amigo_assets::AssetKey;
    use amigo_math::Transform3;
    use amigo_scene::{Mesh3dSceneCommand, SceneEntityId, SceneService};

    #[test]
    fn stores_mesh_draw_commands() {
        let service = MeshSceneService::default();

        service.queue(MeshDrawCommand {
            entity_id: SceneEntityId::new(11),
            entity_name: "playground-3d-probe".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
                transform: Transform3::default(),
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

    #[test]
    fn queues_mesh_scene_command() {
        let scene = SceneService::default();
        let service = MeshSceneService::default();

        let entity = queue_mesh_scene_command(
            &scene,
            &service,
            &Mesh3dSceneCommand::new(
                "playground-3d",
                "playground-3d-probe",
                AssetKey::new("playground-3d/meshes/probe"),
            ),
        );

        assert_eq!(entity.raw(), 0);
        assert_eq!(service.commands().len(), 1);
        assert_eq!(scene.entity_names(), vec!["playground-3d-probe".to_owned()]);
    }
}
