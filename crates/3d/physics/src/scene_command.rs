use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneService, format_scene_command};

use crate::Physics3dSceneService;
use crate::model::{
    BoxCollider3d, BoxCollider3dCommand, PhysicsSpawner3d, PhysicsSpawner3dCommand, PhysicsWorld3d,
    RigidBody3d, RigidBody3dCommand, StaticBoxCollider3d, StaticBoxCollider3dCommand,
};

pub struct Physics3dSceneCommandHandler;

pub fn can_handle_physics3d_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if matches!(
                command.command_type.as_str(),
                amigo_scene::RIGID_BODY_3D_PLUGIN_SCENE_COMMAND_TYPE
                    | amigo_scene::BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE
                    | amigo_scene::STATIC_BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE
                    | amigo_scene::PHYSICS_SPAWNER_3D_PLUGIN_SCENE_COMMAND_TYPE
                    | amigo_scene::PHYSICS_WORLD_3D_PLUGIN_SCENE_COMMAND_TYPE
            )
    )
}

pub fn queue_world_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics3dSceneService,
    command: &amigo_scene::PhysicsWorld3dSceneCommand,
) {
    scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.configure_world(PhysicsWorld3d {
        gravity: command.gravity,
        substeps: command.substeps,
        solver_iterations: command.solver_iterations,
        ccd_substeps: command.ccd_substeps,
    });
}

pub fn queue_rigid_body_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics3dSceneService,
    command: &amigo_scene::RigidBody3dSceneCommand,
) {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_rigid_body(RigidBody3dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        body: RigidBody3d {
            velocity: command.velocity,
            angular_velocity: command.angular_velocity,
            mass: command.mass,
            linear_damping: command.linear_damping,
            angular_damping: command.angular_damping,
            gravity_scale: command.gravity_scale,
            restitution: command.restitution,
            friction: command.friction,
            ccd: command.ccd,
        },
    });
}

pub fn queue_spawner_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics3dSceneService,
    command: &amigo_scene::PhysicsSpawner3dSceneCommand,
) {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_spawner(PhysicsSpawner3dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        source_mod: command.source_mod.clone(),
        spawner: PhysicsSpawner3d {
            entity_prefix: command.entity_prefix.clone(),
            mesh: command.mesh.as_str().to_owned(),
            material: command.material.as_str().to_owned(),
            material_label: command.material_label.clone(),
            interval_seconds: command.interval_seconds,
            origin: command.origin,
            spawn_scale: command.spawn_scale,
            grid_spacing: command.grid_spacing,
            initial_velocity: command.initial_velocity,
            angular_velocity: command.angular_velocity,
            spawn_position_jitter: command.spawn_position_jitter,
            spawn_rotation_jitter: command.spawn_rotation_jitter,
            initial_velocity_jitter: command.initial_velocity_jitter,
            angular_velocity_jitter: command.angular_velocity_jitter,
            mass: command.mass,
            linear_damping: command.linear_damping,
            angular_damping: command.angular_damping,
            gravity_scale: command.gravity_scale,
            restitution: command.restitution,
            friction: command.friction,
            ccd: command.ccd,
            collider_size: command.collider_size,
            max_alive: command.max_alive,
            counter_entity: command.counter_entity.clone(),
            counter_prefix: command.counter_prefix.clone(),
            counter_font: command
                .counter_font
                .as_ref()
                .map(|font| font.as_str().to_owned())
                .unwrap_or_default(),
            counter_size: command.counter_size,
            counter_position: command.counter_position,
        },
    });
}

pub fn queue_box_collider_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics3dSceneService,
    command: &amigo_scene::BoxCollider3dSceneCommand,
) {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_box_collider(BoxCollider3dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        collider: BoxCollider3d {
            size: command.size,
            offset: command.offset,
        },
    });
}

pub fn queue_static_box_collider_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics3dSceneService,
    command: &amigo_scene::StaticBoxCollider3dSceneCommand,
) {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_static_box_collider(StaticBoxCollider3dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        collider: StaticBoxCollider3d {
            size: command.size,
            offset: command.offset,
            friction: command.friction,
            restitution: command.restitution,
        },
    });
}

impl amigo_scene::RuntimeSceneCommandHandler for Physics3dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_physics3d_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let physics_scene_service = runtime.required::<Physics3dSceneService>()?;

        match command {
            SceneCommand::Plugin { command }
                if command.command_type
                    == amigo_scene::PHYSICS_WORLD_3D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let Some(command) = command
                    .payload_as::<amigo_scene::PhysicsWorld3dSceneCommand>()
                    .cloned()
                else {
                    return Err(AmigoError::Message(
                        "physics-3d world plugin command payload mismatch".to_owned(),
                    ));
                };
                queue_world_scene_command(
                    scene_service.as_ref(),
                    physics_scene_service.as_ref(),
                    &command,
                );
            }
            SceneCommand::Plugin { command }
                if command.command_type == amigo_scene::RIGID_BODY_3D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let Some(command) = command
                    .payload_as::<amigo_scene::RigidBody3dSceneCommand>()
                    .cloned()
                else {
                    return Err(AmigoError::Message(
                        "physics-3d rigid body plugin command payload mismatch".to_owned(),
                    ));
                };
                queue_rigid_body_scene_command(
                    scene_service.as_ref(),
                    physics_scene_service.as_ref(),
                    &command,
                );
            }
            SceneCommand::Plugin { command }
                if command.command_type
                    == amigo_scene::BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let Some(command) = command
                    .payload_as::<amigo_scene::BoxCollider3dSceneCommand>()
                    .cloned()
                else {
                    return Err(AmigoError::Message(
                        "physics-3d box collider plugin command payload mismatch".to_owned(),
                    ));
                };
                queue_box_collider_scene_command(
                    scene_service.as_ref(),
                    physics_scene_service.as_ref(),
                    &command,
                );
            }
            SceneCommand::Plugin { command }
                if command.command_type
                    == amigo_scene::STATIC_BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let Some(command) = command
                    .payload_as::<amigo_scene::StaticBoxCollider3dSceneCommand>()
                    .cloned()
                else {
                    return Err(AmigoError::Message(
                        "physics-3d static box collider plugin command payload mismatch".to_owned(),
                    ));
                };
                queue_static_box_collider_scene_command(
                    scene_service.as_ref(),
                    physics_scene_service.as_ref(),
                    &command,
                );
            }
            SceneCommand::Plugin { command }
                if command.command_type
                    == amigo_scene::PHYSICS_SPAWNER_3D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let Some(command) = command
                    .payload_as::<amigo_scene::PhysicsSpawner3dSceneCommand>()
                    .cloned()
                else {
                    return Err(AmigoError::Message(
                        "physics-3d spawner plugin command payload mismatch".to_owned(),
                    ));
                };
                queue_spawner_scene_command(
                    scene_service.as_ref(),
                    physics_scene_service.as_ref(),
                    &command,
                );
            }
            _ => {
                return Err(AmigoError::Message(format!(
                    "physics-3d cannot handle command {}",
                    format_scene_command(&command)
                )));
            }
        }
        Ok(())
    }
}
