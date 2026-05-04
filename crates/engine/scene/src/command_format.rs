use crate::*;

pub fn format_scene_command(command: &SceneCommand) -> String {
    match command {
        SceneCommand::SpawnNamedEntity { name, .. } => format!("scene.spawn({name})"),
        SceneCommand::ConfigureEntity { entity_name, .. } => {
            format!("scene.configure({entity_name})")
        }
        SceneCommand::SelectScene { scene } => format!("scene.select({})", scene.as_str()),
        SceneCommand::ReloadActiveScene => "scene.reload_active".to_owned(),
        SceneCommand::ClearEntities => "scene.clear".to_owned(),
        SceneCommand::QueueSprite2d { command } => format!(
            "scene.2d.sprite({}, {}, {}x{})",
            command.entity_name,
            command.texture.as_str(),
            command.size.x,
            command.size.y
        ),
        SceneCommand::QueueTileMap2d { command } => format!(
            "scene.2d.tilemap({}, {}, {} rows)",
            command.entity_name,
            command.tileset.as_str(),
            command.grid.len()
        ),
        SceneCommand::QueueText2d { command } => format!(
            "scene.2d.text({}, {}, {}x{})",
            command.entity_name,
            command.font.as_str(),
            command.bounds.x,
            command.bounds.y
        ),
        SceneCommand::QueueVectorShape2d { command } => format!(
            "scene.2d.vector({}, {:?})",
            command.entity_name, command.kind
        ),
        SceneCommand::QueueEntityPool { command } => {
            format!(
                "scene.pool({}, {} members)",
                command.pool,
                command.members.len()
            )
        }
        SceneCommand::QueueLifetime { command } => {
            format!(
                "scene.lifetime({}, {}s)",
                command.entity_name, command.seconds
            )
        }
        SceneCommand::QueueProjectileEmitter2d { command } => format!(
            "scene.2d.projectile_emitter({}, pool={}, speed={})",
            command.entity_name, command.pool, command.speed
        ),
        SceneCommand::QueueInputActionMap { command } => format!(
            "scene.input.action_map({}, {} actions)",
            command.id,
            command.actions.len()
        ),
        SceneCommand::QueueBehavior { command } => {
            format!(
                "scene.behavior({}, {:?})",
                command.entity_name, command.behavior
            )
        }
        SceneCommand::QueueEventPipeline { command } => format!(
            "scene.event.pipeline({}, topic={}, {} steps)",
            command.id,
            command.topic,
            command.steps.len()
        ),
        SceneCommand::QueueUiModelBindings { command } => format!(
            "scene.ui.model_bindings({}, {} bindings)",
            command.entity_name,
            command.bindings.len()
        ),
        SceneCommand::QueueScriptComponent { command } => format!(
            "scene.script_component({}, {})",
            command.entity_name,
            command.script.display()
        ),
        SceneCommand::QueueParticleEmitter2d { command } => format!(
            "scene.2d.particle_emitter({}, spawn_rate={}, lifetime={})",
            command.entity_name, command.spawn_rate, command.particle_lifetime
        ),
        SceneCommand::QueueVelocity2d { command } => format!(
            "scene.2d.velocity({}, {}, {})",
            command.entity_name, command.velocity.x, command.velocity.y
        ),
        SceneCommand::QueueBounds2d { command } => format!(
            "scene.2d.bounds({}, {:?})",
            command.entity_name, command.behavior
        ),
        SceneCommand::QueueFreeflightMotion2d { command } => format!(
            "scene.2d.freeflight({}, max_speed={}, max_angular_speed={})",
            command.entity_name, command.max_speed, command.max_angular_speed
        ),
        SceneCommand::QueueKinematicBody2d { command } => format!(
            "scene.2d.physics.body({}, {}, {}, {})",
            command.entity_name, command.velocity.x, command.velocity.y, command.gravity_scale
        ),
        SceneCommand::QueueAabbCollider2d { command } => format!(
            "scene.2d.physics.collider({}, {}x{}, {})",
            command.entity_name, command.size.x, command.size.y, command.layer
        ),
        SceneCommand::QueueStaticCollider2d { command } => format!(
            "scene.2d.physics.static_collider({}, {}x{}, {})",
            command.entity_name, command.size.x, command.size.y, command.layer
        ),
        SceneCommand::QueueCircleCollider2d { command } => format!(
            "scene.2d.physics.circle({}, r={}, {}, {})",
            command.entity_name, command.radius, command.offset.x, command.offset.y
        ),
        SceneCommand::QueueTrigger2d { command } => format!(
            "scene.2d.physics.trigger({}, {}x{}, {})",
            command.entity_name,
            command.size.x,
            command.size.y,
            command.event.as_deref().unwrap_or("none")
        ),
        SceneCommand::QueueCollisionEventRule2d { command } => format!(
            "scene.2d.physics.collision_event({}, {})",
            command.id, command.event
        ),
        SceneCommand::QueueMotionController2d { command } => format!(
            "scene.2d.motion({}, max_speed={}, jump_velocity={})",
            command.entity_name, command.max_speed, command.jump_velocity
        ),
        SceneCommand::QueueCameraFollow2d { command } => format!(
            "scene.2d.camera_follow({}, {}, {}, {})",
            command.entity_name, command.target, command.offset.x, command.offset.y
        ),
        SceneCommand::QueueParallax2d { command } => format!(
            "scene.2d.parallax({}, {}, {}, {})",
            command.entity_name, command.camera, command.factor.x, command.factor.y
        ),
        SceneCommand::QueueTileMapMarker2d { command } => format!(
            "scene.2d.tilemap_marker({}, {}, #{})",
            command.entity_name, command.symbol, command.index
        ),
        SceneCommand::QueueMesh3d { command } => format!(
            "scene.3d.mesh({}, {})",
            command.entity_name,
            command.mesh_asset.as_str()
        ),
        SceneCommand::QueueMaterial3d { command } => format!(
            "scene.3d.material({}, {}, {})",
            command.entity_name,
            command.label,
            command
                .source
                .as_ref()
                .map(|asset| asset.as_str().to_owned())
                .unwrap_or_else(|| "generated".to_owned())
        ),
        SceneCommand::QueueText3d { command } => format!(
            "scene.3d.text({}, {}, {})",
            command.entity_name,
            command.font.as_str(),
            command.size
        ),
        SceneCommand::QueueUi { command } => {
            format!("scene.ui({}, screen-space)", command.entity_name)
        }
        SceneCommand::QueueUiThemeSet { command } => format!(
            "scene.ui.theme_set({}, {} themes)",
            command.entity_name,
            command.themes.len()
        ),
        SceneCommand::QueueAudioCue { command } => {
            format!(
                "scene.audio.cue({}, {})",
                command.name,
                command.clip.as_str()
            )
        }
        SceneCommand::QueueActivationSet { command } => {
            format!(
                "scene.activation_set({}, {} entries)",
                command.id,
                command.entries.len()
            )
        }
        SceneCommand::ActivateSet { id } => format!("scene.activate_set({id})"),
    }
}
