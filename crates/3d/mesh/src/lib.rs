//! 3D mesh scene service for referencing authored geometry.
//! It stores mesh bindings that the renderer resolves into GPU-ready draw data.

use std::collections::BTreeMap;
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
    npr_presets: Mutex<BTreeMap<String, amigo_render_api::NprLineSettings3d>>,
}

impl MeshSceneService {
    pub fn queue(&self, command: MeshDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
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

    pub fn register_npr_preset(
        &self,
        id: impl Into<String>,
        settings: amigo_render_api::NprLineSettings3d,
    ) {
        let mut presets = self
            .npr_presets
            .lock()
            .expect("mesh NPR preset mutex should not be poisoned");
        presets.insert(id.into(), settings);
    }

    pub fn npr_preset(&self, id: &str) -> Option<amigo_render_api::NprLineSettings3d> {
        let presets = self
            .npr_presets
            .lock()
            .expect("mesh NPR preset mutex should not be poisoned");
        presets.get(id).cloned()
    }

    pub fn npr_preset_ids(&self) -> Vec<String> {
        let presets = self
            .npr_presets
            .lock()
            .expect("mesh NPR preset mutex should not be poisoned");
        presets.keys().cloned().collect()
    }

    pub fn apply_npr_preset(&self, entity_name: &str, preset_id: &str) -> bool {
        let Some(settings) = self.npr_preset(preset_id) else {
            return false;
        };
        let mut commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        command.mesh.npr = Some(settings);
        true
    }

    pub fn set_npr_temporal_path_smoothing(&self, entity_name: &str, enabled: bool) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("mesh scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        let Some(npr) = command.mesh.npr.as_mut() else {
            return false;
        };
        npr.temporal_path_smoothing = enabled;
        true
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
        plugin_scene_handlers.register(
            amigo_scene::NPR_PRESET_3D_PLUGIN_SCENE_COMMAND_TYPE,
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
    use amigo_render_api::{NprLineSettings3d, NprRenderStrategy3d};
    use amigo_scene::{Mesh3dSceneCommand, SceneService};
    use amigo_scripting_api::ScriptCommand;

    use crate::{
        Mesh3dScriptCommandContext, Mesh3dScriptCommandOutcome, handle_mesh3d_script_command,
    };

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
    fn queues_mesh_draw_commands_by_entity_name() {
        let service = MeshSceneService::default();
        let entity_name = "playground-npr-model".to_owned();

        service.queue(MeshDrawCommand {
            entity_id: 11,
            entity_name: entity_name.clone(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-npr/meshes/first"),
                transform: Transform3::default(),
                npr: None,
            },
        });
        service.queue(MeshDrawCommand {
            entity_id: 11,
            entity_name,
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-npr/meshes/second"),
                transform: Transform3::default(),
                npr: None,
            },
        });

        let commands = service.commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0].mesh.mesh_asset,
            AssetKey::new("playground-npr/meshes/second")
        );
    }

    #[test]
    fn applies_registered_npr_preset_to_mesh_command() {
        let service = MeshSceneService::default();
        service.queue(MeshDrawCommand {
            entity_id: 11,
            entity_name: "playground-npr-model".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-npr/meshes/soldier"),
                transform: Transform3::default(),
                npr: None,
            },
        });
        service.register_npr_preset(
            "heavy_noir_ink",
            NprLineSettings3d {
                width_px: 4.0,
                seed: 9090,
                ..NprLineSettings3d::default()
            },
        );

        assert!(service.apply_npr_preset("playground-npr-model", "heavy_noir_ink"));

        let npr = service.commands()[0]
            .mesh
            .npr
            .as_ref()
            .expect("preset should set NPR settings")
            .clone();
        assert_eq!(npr.width_px, 4.0);
        assert_eq!(npr.seed, 9090);
    }

    #[test]
    fn applies_npr_preset_with_cpu_reference_strategy() {
        let service = MeshSceneService::default();
        service.queue(MeshDrawCommand {
            entity_id: 11,
            entity_name: "playground-npr-model".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-npr/meshes/soldier"),
                transform: Transform3::default(),
                npr: None,
            },
        });
        service.register_npr_preset(
            "cpu_ref",
            NprLineSettings3d {
                render_strategy: NprRenderStrategy3d::CpuReference,
                ..NprLineSettings3d::default()
            },
        );

        assert!(service.apply_npr_preset("playground-npr-model", "cpu_ref"));
        assert_eq!(
            service.commands()[0]
                .mesh
                .npr
                .as_ref()
                .expect("preset should apply strategy")
                .render_strategy,
            NprRenderStrategy3d::CpuReference
        );
    }

    #[test]
    fn script_command_applies_npr_preset_to_mesh_command() {
        let service = MeshSceneService::default();
        service.queue(MeshDrawCommand {
            entity_id: 11,
            entity_name: "playground-npr-model".to_owned(),
            mesh: Mesh3d {
                mesh_asset: AssetKey::new("playground-npr/meshes/soldier"),
                transform: Transform3::default(),
                npr: None,
            },
        });
        service.register_npr_preset(
            "loose_pencil",
            NprLineSettings3d {
                humanization: 0.82,
                ..NprLineSettings3d::default()
            },
        );

        let outcome = handle_mesh3d_script_command(
            Mesh3dScriptCommandContext {
                selected_mod: "playground-npr",
                mesh_scene_service: Some(&service),
            },
            ScriptCommand::new(
                "3d.mesh",
                "apply_npr_preset",
                vec!["playground-npr-model".to_owned(), "loose_pencil".to_owned()],
            ),
        );

        assert!(matches!(
            outcome,
            Mesh3dScriptCommandOutcome::AppliedNprPreset { .. }
        ));
        assert_eq!(
            service.commands()[0]
                .mesh
                .npr
                .as_ref()
                .expect("script command should apply preset")
                .humanization,
            0.82
        );
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
            "playground-npr-model-1-soldier",
            AssetKey::new("playground-npr/meshes/soldier"),
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
    fn queues_mesh_scene_command_with_npr_gpu_strategy() {
        let scene = SceneService::default();
        let service = MeshSceneService::default();
        let mut command = Mesh3dSceneCommand::new(
            "playground-npr",
            "playground-npr-model-1-soldier",
            AssetKey::new("playground-npr/meshes/soldier"),
        );
        command.npr = Some(NprLineSettings3d {
            render_strategy: NprRenderStrategy3d::GpuRealtime,
            ..NprLineSettings3d::default()
        });

        queue_mesh_scene_command(&scene, &service, &command);

        assert_eq!(
            service.commands()[0]
                .mesh
                .npr
                .as_ref()
                .expect("npr line settings should be preserved")
                .render_strategy,
            NprRenderStrategy3d::GpuRealtime
        );
    }

    #[test]
    fn mesh_editor_capability_uses_mesh3d_component_type() {
        let capability = Mesh3dEditorCapability;
        assert_eq!(capability.component_type().as_str(), "amigo.3d.mesh");
        assert_eq!(capability.inspector_schema().fields.len(), 4);
    }
}
