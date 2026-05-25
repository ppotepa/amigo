//! 3D material scene service for mesh rendering.
//! It binds authored material parameters to entities so the render backend can shade 3D content.

use std::sync::Mutex;

use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
pub use amigo_render_api::{Material3d, MaterialDrawCommand};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{Material3dSceneCommand, SceneEntityId, SceneService};
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
        amigo_scene::register_scene_reset_handler(registry, MaterialSceneResetHandler)?;
        registry.register(MaterialDomainInfo {
            crate_name: "amigo-3d-material",
            capability: "materials_3d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-3d-material",
            &["materials_3d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let plugin_scene_handlers =
            registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        plugin_scene_handlers.register(
            amigo_scene::MATERIAL_3D_PLUGIN_SCENE_COMMAND_TYPE,
            std::sync::Arc::new(crate::scene_command::Material3dSceneCommandHandler),
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::Material3dScriptCommandHandler,
        );
        Ok(())
    }
}

pub fn queue_material_scene_command(
    scene_service: &SceneService,
    material_scene_service: &MaterialSceneService,
    command: &Material3dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    material_scene_service.queue(MaterialDrawCommand {
        entity_id: entity.raw(),
        entity_name: command.entity_name.clone(),
        material: Material3d {
            label: command.label.clone(),
            albedo: command.albedo,
            source: command.source.clone(),
        },
    });
    entity
}

#[cfg(test)]
mod tests {
    use super::{
        queue_material_scene_command, Material3d, Material3dEditorCapability, MaterialDrawCommand,
        MaterialSceneService,
    };
    use amigo_assets::AssetKey;
    use amigo_editor_api::EditorCapability;
    use amigo_math::ColorRgba;
    use amigo_scene::{Material3dSceneCommand, SceneService};

    #[test]
    fn stores_material_draw_commands() {
        let service = MaterialSceneService::default();

        service.queue(MaterialDrawCommand {
            entity_id: 13,
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

    #[test]
    fn queues_material_scene_command() {
        let scene = SceneService::default();
        let service = MaterialSceneService::default();

        let entity = queue_material_scene_command(
            &scene,
            &service,
            &Material3dSceneCommand::new(
                "playground-3d",
                "playground-3d-probe",
                "debug-surface",
                Some(AssetKey::new("playground-3d/materials/debug-surface")),
            ),
        );

        assert_eq!(entity.raw(), 0);
        assert_eq!(service.commands().len(), 1);
        assert_eq!(scene.entity_names(), vec!["playground-3d-probe".to_owned()]);
    }

    #[test]
    fn material_editor_capability_uses_material3d_component_type() {
        let capability = Material3dEditorCapability;
        assert_eq!(capability.component_type().as_str(), "amigo.3d.material");
        assert_eq!(capability.inspector_schema().fields.len(), 4);
    }
}
