use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
};
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{MeshSceneService, queue_mesh_scene_command};

pub struct Mesh3dSceneCommandHandler;

pub struct MeshSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub mesh_scene_service: &'a MeshSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct MeshSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub mesh_asset: AssetKey,
}

pub fn can_handle_mesh_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::MESH_3D_PLUGIN_SCENE_COMMAND_TYPE
    )
}

pub fn handle_mesh_scene_command(
    ctx: MeshSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<MeshSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::MESH_3D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let Some(command) = command
                .payload_as::<amigo_scene::Mesh3dSceneCommand>()
                .cloned()
            else {
                return Err(AmigoError::Message(
                    "mesh-3d plugin command payload mismatch".to_owned(),
                ));
            };
            let entity =
                queue_mesh_scene_command(ctx.scene_service, ctx.mesh_scene_service, &command);
            ctx.scene_event_queue.publish(SceneEvent::MeshQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                mesh_asset: command.mesh_asset.clone(),
            });
            Ok(MeshSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                mesh_asset: command.mesh_asset,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "mesh-3d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Mesh3dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::MESH_3D_PLUGIN_SCENE_COMMAND_TYPE
            || command.command_type == amigo_scene::MESH_ANIMATION_3D_PLUGIN_SCENE_COMMAND_TYPE)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        if let SceneCommand::Plugin {
            command: plugin_command,
        } = &command
        {
            if let Some(animation) = plugin_command
                .payload_as::<amigo_scene::MeshAnimation3dSceneCommand>()
            {
                let meshes = runtime.required::<MeshSceneService>()?;
                meshes.play_animation_sampled(
                    animation.entity_name.clone(),
                    animation.clip.clone(),
                    animation.speed,
                    animation.looped,
                    animation.sample_fps,
                ).map_err(AmigoError::Message)?;
                return Ok(());
            }
            if let Some(mesh_command) =
                plugin_command.payload_as::<amigo_scene::Mesh3dSceneCommand>()
            {
                if let Some(asset_catalog) = runtime.resolve::<AssetCatalog>() {
                    asset_catalog.register_manifest(AssetManifest {
                        key: mesh_command.mesh_asset.clone(),
                        source: AssetSourceKind::Mod(mesh_command.source_mod.clone()),
                        tags: vec!["mesh-3d".to_owned(), "gltf".to_owned()],
                    });
                    asset_catalog.request_load(AssetLoadRequest::new(
                        mesh_command.mesh_asset.clone(),
                        AssetLoadPriority::Interactive,
                    ));
                }
            }
        }
        let scene_service = runtime.required::<SceneService>()?;
        let mesh_scene_service = runtime.required::<MeshSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_mesh_scene_command(
            MeshSceneCommandContext {
                scene_service: scene_service.as_ref(),
                mesh_scene_service: mesh_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
