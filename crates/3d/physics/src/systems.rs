use std::sync::Arc;

use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use amigo_math::{ColorRgba, Transform3, Vec3};
use amigo_runtime::Runtime;
use amigo_scene::{
    BoxCollider3dSceneCommand, Material3dSceneCommand, Mesh3dSceneCommand, RigidBody3dSceneCommand,
    SceneCommand, SceneCommandQueue, SceneService, Text3dSceneCommand,
};
use glam::EulerRot;
use rapier3d::prelude::*;

use crate::{Physics3dSceneService, registry::Physics3dState};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

pub fn tick_physics_3d(runtime: &Runtime) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let physics_scene_service = required::<Physics3dSceneService>(runtime)?;
    let delta_seconds = amigo_session::simulation_delta_seconds(runtime);
    if delta_seconds <= 0.0 {
        return Ok(());
    }

    tick_spawners(runtime, physics_scene_service.as_ref(), delta_seconds)?;

    physics_scene_service.with_state_mut(|state| {
        sync_authored_bodies_to_rapier(state, scene_service.as_ref());
        step_rapier_world(state, delta_seconds);
        writeback_rapier_transforms(state, scene_service.as_ref());
    });

    Ok(())
}

fn sync_authored_bodies_to_rapier(state: &mut Physics3dState, scene_service: &SceneService) {
    for body_command in state.rigid_bodies.values() {
        if state
            .rigid_body_handles
            .contains_key(&body_command.entity_name)
        {
            continue;
        }
        let transform = scene_service
            .transform_of(&body_command.entity_name)
            .unwrap_or_default();
        let body = &body_command.body;
        let rigid_body = RigidBodyBuilder::dynamic()
            .pose(isometry_from_transform(transform))
            .linvel(Vector::new(
                body.velocity.x,
                body.velocity.y,
                body.velocity.z,
            ))
            .angvel(Vector::new(
                body.angular_velocity.x,
                body.angular_velocity.y,
                body.angular_velocity.z,
            ))
            .additional_mass(body.mass.max(0.001))
            .gravity_scale(body.gravity_scale)
            .linear_damping(body.linear_damping.max(0.0))
            .angular_damping(body.angular_damping.max(0.0))
            .ccd_enabled(body.ccd)
            .build();
        let handle = state.rapier.rigid_bodies.insert(rigid_body);
        state
            .rigid_body_handles
            .insert(body_command.entity_name.clone(), handle);
    }

    for collider_command in state.box_colliders.values() {
        if state
            .collider_handles
            .contains_key(&collider_command.entity_name)
        {
            continue;
        }
        let Some(body_handle) = state
            .rigid_body_handles
            .get(&collider_command.entity_name)
            .copied()
        else {
            continue;
        };
        let body = state.rigid_bodies.get(&collider_command.entity_name);
        let collider = ColliderBuilder::cuboid(
            half_extent(collider_command.collider.size.x),
            half_extent(collider_command.collider.size.y),
            half_extent(collider_command.collider.size.z),
        )
        .translation(Vector::new(
            collider_command.collider.offset.x,
            collider_command.collider.offset.y,
            collider_command.collider.offset.z,
        ))
        .restitution(body.map(|body| body.body.restitution).unwrap_or_default())
        .friction(body.map(|body| body.body.friction).unwrap_or(0.8))
        .build();
        let collider_handle = state.rapier.colliders.insert_with_parent(
            collider,
            body_handle,
            &mut state.rapier.rigid_bodies,
        );
        state
            .collider_handles
            .insert(collider_command.entity_name.clone(), vec![collider_handle]);
    }

    for static_collider in &state.static_box_colliders {
        if state
            .rigid_body_handles
            .contains_key(&static_collider.entity_name)
        {
            continue;
        }
        let fixed_body = RigidBodyBuilder::fixed()
            .translation(Vector::new(
                static_collider.collider.offset.x,
                static_collider.collider.offset.y,
                static_collider.collider.offset.z,
            ))
            .build();
        let body_handle = state.rapier.rigid_bodies.insert(fixed_body);
        let collider = ColliderBuilder::cuboid(
            half_extent(static_collider.collider.size.x),
            half_extent(static_collider.collider.size.y),
            half_extent(static_collider.collider.size.z),
        )
        .friction(static_collider.collider.friction.max(0.0))
        .restitution(static_collider.collider.restitution.max(0.0))
        .build();
        let collider_handle = state.rapier.colliders.insert_with_parent(
            collider,
            body_handle,
            &mut state.rapier.rigid_bodies,
        );
        state
            .rigid_body_handles
            .insert(static_collider.entity_name.clone(), body_handle);
        state
            .collider_handles
            .insert(static_collider.entity_name.clone(), vec![collider_handle]);
    }
}

fn step_rapier_world(state: &mut Physics3dState, delta_seconds: f32) {
    let rapier = &mut state.rapier;
    let world = &state.world;
    rapier.gravity = Vector::new(world.gravity.x, world.gravity.y, world.gravity.z);
    rapier.integration_parameters.num_solver_iterations =
        world.solver_iterations.clamp(1, 64) as usize;
    rapier.integration_parameters.max_ccd_substeps = world.ccd_substeps.clamp(1, 32) as usize;

    let substeps = world.substeps.clamp(1, 16);
    let step_dt = (delta_seconds.min(1.0 / 15.0) / substeps as f32).max(0.000_1);
    for _ in 0..substeps {
        rapier.integration_parameters.dt = step_dt;
        rapier.pipeline.step(
            rapier.gravity,
            &rapier.integration_parameters,
            &mut rapier.island_manager,
            &mut rapier.broad_phase,
            &mut rapier.narrow_phase,
            &mut rapier.rigid_bodies,
            &mut rapier.colliders,
            &mut rapier.impulse_joints,
            &mut rapier.multibody_joints,
            &mut rapier.ccd_solver,
            &(),
            &(),
        );
    }
}

fn writeback_rapier_transforms(state: &mut Physics3dState, scene_service: &SceneService) {
    for body_command in state.rigid_bodies.values() {
        let Some(handle) = state
            .rigid_body_handles
            .get(&body_command.entity_name)
            .copied()
        else {
            continue;
        };
        let Some(body) = state.rapier.rigid_bodies.get(handle) else {
            continue;
        };
        let transform = transform_from_isometry(*body.position());
        scene_service.set_transform(&body_command.entity_name, transform);
        let linvel = body.linvel();
        let angvel = body.angvel();
        state.body_states.insert(
            body_command.entity_name.clone(),
            crate::PhysicsBodyState3d {
                velocity: Vec3::new(linvel.x, linvel.y, linvel.z),
                angular_velocity: Vec3::new(angvel.x, angvel.y, angvel.z),
                grounded: false,
            },
        );
    }
}

fn isometry_from_transform(transform: Transform3) -> Pose {
    Pose::from_parts(
        Vector::new(
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        ),
        Rotation::from_euler(
            EulerRot::XYZ,
            transform.rotation_euler.x,
            transform.rotation_euler.y,
            transform.rotation_euler.z,
        ),
    )
}

fn transform_from_isometry(isometry: Pose) -> Transform3 {
    let (x, y, z) = isometry.rotation.to_euler(EulerRot::XYZ);
    Transform3 {
        translation: Vec3::new(
            isometry.translation.x,
            isometry.translation.y,
            isometry.translation.z,
        ),
        rotation_euler: Vec3::new(x, y, z),
        scale: Vec3::new(1.0, 1.0, 1.0),
    }
}

fn half_extent(size: f32) -> f32 {
    (size * 0.5).max(0.001)
}

fn tick_spawners(
    runtime: &Runtime,
    physics_scene_service: &Physics3dSceneService,
    delta_seconds: f32,
) -> AmigoResult<()> {
    let spawners = physics_scene_service.spawners();
    if spawners.is_empty() {
        return Ok(());
    }
    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;

    for command in spawners {
        let interval = command.spawner.interval_seconds.max(0.001);
        let mut state = physics_scene_service.spawner_state(&command.entity_name);
        state.elapsed_seconds += delta_seconds;
        let mut spawned = false;

        while state.elapsed_seconds >= interval
            && (command.spawner.max_alive == 0 || state.spawn_index < command.spawner.max_alive)
        {
            state.elapsed_seconds -= interval;
            let index = state.spawn_index;
            state.spawn_index += 1;
            submit_spawned_cube(scene_command_queue.as_ref(), &command, index);
            spawned = true;
        }
        if spawned {
            submit_spawn_counter(scene_command_queue.as_ref(), &command, state.spawn_index);
        }

        physics_scene_service.sync_spawner_state(&command.entity_name, state);
    }

    Ok(())
}

fn submit_spawned_cube(
    scene_command_queue: &SceneCommandQueue,
    command: &crate::PhysicsSpawner3dCommand,
    index: u32,
) {
    let entity_name = format!("{}-{}", command.spawner.entity_prefix, index);
    let mut transform = Transform3::default();
    transform.translation = spawn_translation(command, index);
    transform.rotation_euler = spawn_rotation(command, index);
    transform.scale = command.spawner.spawn_scale;

    scene_command_queue.submit(SceneCommand::SpawnNamedEntity {
        name: entity_name.clone(),
        transform: Some(transform),
    });
    scene_command_queue.submit(SceneCommand::Plugin {
        command: amigo_scene::mesh_3d_plugin_scene_command(Mesh3dSceneCommand {
            source_mod: command.source_mod.clone(),
            entity_name: entity_name.clone(),
            mesh_asset: AssetKey::new(command.spawner.mesh.clone()),
            transform,
            npr: None,
        }),
    });
    scene_command_queue.submit(SceneCommand::Plugin {
        command: amigo_scene::material_3d_plugin_scene_command(Material3dSceneCommand {
            source_mod: command.source_mod.clone(),
            entity_name: entity_name.clone(),
            label: command.spawner.material_label.clone(),
            albedo: ColorRgba::WHITE,
            source: Some(AssetKey::new(command.spawner.material.clone())),
            render_order: 0,
        }),
    });
    scene_command_queue.submit(SceneCommand::Plugin {
        command: amigo_scene::rigid_body_3d_plugin_scene_command(
            RigidBody3dSceneCommand::new(
                command.source_mod.clone(),
                entity_name.clone(),
                spawn_initial_velocity(command.spawner.initial_velocity, command, index),
                command.spawner.gravity_scale,
                command.spawner.restitution,
            )
            .with_angular(
                spawn_angular_velocity(command.spawner.angular_velocity, command, index),
                command.spawner.angular_damping,
            )
            .with_physical_properties(
                command.spawner.mass,
                command.spawner.linear_damping,
                command.spawner.friction,
                command.spawner.ccd,
            ),
        ),
    });
    scene_command_queue.submit(SceneCommand::Plugin {
        command: amigo_scene::box_collider_3d_plugin_scene_command(BoxCollider3dSceneCommand::new(
            command.source_mod.clone(),
            entity_name,
            command.spawner.collider_size,
            Vec3::ZERO,
        )),
    });
}

fn submit_spawn_counter(
    scene_command_queue: &SceneCommandQueue,
    command: &crate::PhysicsSpawner3dCommand,
    count: u32,
) {
    let Some(entity_name) = command.spawner.counter_entity.as_ref() else {
        return;
    };
    if command.spawner.counter_font.trim().is_empty() {
        return;
    }

    let mut transform = Transform3::default();
    transform.translation = command.spawner.counter_position;
    scene_command_queue.submit(SceneCommand::Plugin {
        command: amigo_scene::text_3d_plugin_scene_command(Text3dSceneCommand {
            source_mod: command.source_mod.clone(),
            entity_name: entity_name.clone(),
            content: format!("{}{}", command.spawner.counter_prefix, count),
            font: AssetKey::new(command.spawner.counter_font.clone()),
            size: command.spawner.counter_size,
            transform,
        }),
    });
}

fn spawn_translation(command: &crate::PhysicsSpawner3dCommand, index: u32) -> Vec3 {
    let lane = (index % 3) as f32 - 1.0;
    let row = ((index / 3) % 3) as f32 - 1.0;
    let stack = (index / 9) as f32;
    add_vec3(
        Vec3::new(
            command.spawner.origin.x + lane * command.spawner.grid_spacing.x,
            command.spawner.origin.y + stack * command.spawner.grid_spacing.y,
            command.spawner.origin.z + row * command.spawner.grid_spacing.z,
        ),
        deterministic_jitter(command.spawner.spawn_position_jitter, index, 11),
    )
}

fn spawn_rotation(command: &crate::PhysicsSpawner3dCommand, index: u32) -> Vec3 {
    let index_f = index as f32 + 1.0;
    add_vec3(
        Vec3::new(index_f * 0.37, index_f * 0.61, index_f * 0.23),
        deterministic_jitter(command.spawner.spawn_rotation_jitter, index, 23),
    )
}

fn spawn_angular_velocity(
    base: Vec3,
    command: &crate::PhysicsSpawner3dCommand,
    index: u32,
) -> Vec3 {
    let index_f = index as f32 + 1.0;
    add_vec3(
        Vec3::new(
            base.x + 0.08 * index_f,
            base.y + 0.05 * index_f,
            base.z + 0.03 * index_f,
        ),
        deterministic_jitter(command.spawner.angular_velocity_jitter, index, 41),
    )
}

fn spawn_initial_velocity(
    base: Vec3,
    command: &crate::PhysicsSpawner3dCommand,
    index: u32,
) -> Vec3 {
    add_vec3(
        base,
        deterministic_jitter(command.spawner.initial_velocity_jitter, index, 29),
    )
}

fn add_vec3(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x + right.x, left.y + right.y, left.z + right.z)
}

fn deterministic_jitter(amplitude: Vec3, index: u32, salt: u32) -> Vec3 {
    Vec3::new(
        signed_noise(index, salt) * amplitude.x,
        signed_noise(index, salt + 17) * amplitude.y,
        signed_noise(index, salt + 31) * amplitude.z,
    )
}

fn signed_noise(index: u32, salt: u32) -> f32 {
    let mut value = index
        .wrapping_mul(1_664_525)
        .wrapping_add(salt.wrapping_mul(1_013_904_223));
    value ^= value >> 16;
    value = value.wrapping_mul(2_246_822_519);
    let unit = (value & 0xffff) as f32 / 65_535.0;
    unit * 2.0 - 1.0
}
