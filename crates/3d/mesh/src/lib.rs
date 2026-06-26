//! 3D mesh scene service for referencing authored geometry.
//! It stores mesh bindings that the renderer resolves into GPU-ready draw data.

use std::sync::Mutex;

use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
pub use amigo_render_api::{Mesh3d, MeshDrawCommand};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{Mesh3dSceneCommand, SceneEntityId, SceneService};
mod editor_capability;
mod render_extraction;
mod reset;
mod runtime_capabilities;
mod scene_command;
mod script_command;
pub use editor_capability::*;
pub use render_extraction::*;
pub use reset::*;
pub use runtime_capabilities::*;
pub use scene_command::*;
pub use script_command::*;

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
        amigo_scene::register_scene_reset_handler(registry, MeshSceneResetHandler)?;
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
        )?;
        let plugin_scene_handlers =
            registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        plugin_scene_handlers.register(
            amigo_scene::MESH_3D_PLUGIN_SCENE_COMMAND_TYPE,
            std::sync::Arc::new(crate::scene_command::Mesh3dSceneCommandHandler),
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::Mesh3dScriptCommandHandler,
        );
        Ok(())
    }
}

pub fn queue_mesh_scene_command(
    scene_service: &SceneService,
    mesh_scene_service: &MeshSceneService,
    command: &Mesh3dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    mesh_scene_service.queue(MeshDrawCommand {
        entity_id: entity.raw(),
        entity_name: command.entity_name.clone(),
        mesh: Mesh3d {
            mesh_asset: command.mesh_asset.clone(),
            transform: command.transform,
            npr: command.npr.clone(),
        },
    });
    entity
}

#[cfg(test)]
mod tests {
    use super::{
        Mesh3d, Mesh3dEditorCapability, MeshDrawCommand, MeshSceneService, queue_mesh_scene_command,
    };
    use amigo_assets::AssetKey;
    use amigo_editor_api::EditorCapability;
    use amigo_math::Transform3;
    use amigo_render_api::NprLineSettings3d;
    use amigo_scene::{Mesh3dSceneCommand, SceneService};

    #[test]
    fn stores_mesh_draw_commands() {
        let service = MeshSceneService::default();

        service.queue(MeshDrawCommand {
            entity_id: 11,
            entity_name: "playground-3d-probe".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
                transform: Transform3::default(),
                npr: None,
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

    #[test]
    fn queues_mesh_scene_command_with_npr_line_settings() {
        let scene = SceneService::default();
        let service = MeshSceneService::default();
        let mut command = Mesh3dSceneCommand::new(
            "playground-npr",
            "playground-npr-box-source",
            AssetKey::new("playground-npr/meshes/box-source"),
        );
        command.npr = Some(NprLineSettings3d {
            feature_angle_degrees: 30.0,
            seed: 2602,
            ..NprLineSettings3d::default()
        });

        queue_mesh_scene_command(&scene, &service, &command);

        let queued = service.commands();
        assert_eq!(queued.len(), 1);
        let npr = queued[0]
            .mesh
            .npr
            .as_ref()
            .expect("npr line settings should be preserved");
        assert_eq!(npr.feature_angle_degrees, 30.0);
        assert_eq!(npr.seed, 2602);
    }

    #[test]
    fn mesh_editor_capability_uses_mesh3d_component_type() {
        let capability = Mesh3dEditorCapability;
        assert_eq!(capability.component_type().as_str(), "amigo.3d.mesh");
        assert_eq!(capability.inspector_schema().fields.len(), 4);
    }
}
