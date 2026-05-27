pub const RIGID_BODY_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.3d.scene-command.RigidBody3d";
pub const BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.3d.scene-command.BoxCollider3d";
pub const STATIC_BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.3d.scene-command.StaticBoxCollider3d";
pub const PHYSICS_SPAWNER_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.3d.scene-command.PhysicsSpawner3d";
pub const PHYSICS_WORLD_3D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.3d.scene-command.PhysicsWorld3d";

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsWorld3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub gravity: Vec3,
    pub substeps: u32,
    pub solver_iterations: u32,
    pub ccd_substeps: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsWorld3dPluginSceneCommandPayload(pub PhysicsWorld3dSceneCommand);

impl crate::PluginSceneCommandPayload for PhysicsWorld3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        PHYSICS_WORLD_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<PhysicsWorld3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn physics_world_3d_plugin_scene_command(
    command: PhysicsWorld3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(PhysicsWorld3dPluginSceneCommandPayload(
        command,
    )))
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigidBody3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub restitution: f32,
    pub friction: f32,
    pub ccd: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigidBody3dPluginSceneCommandPayload(pub RigidBody3dSceneCommand);

impl crate::PluginSceneCommandPayload for RigidBody3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        RIGID_BODY_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<RigidBody3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn rigid_body_3d_plugin_scene_command(
    command: RigidBody3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(RigidBody3dPluginSceneCommandPayload(
        command,
    )))
}

impl RigidBody3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        velocity: Vec3,
        gravity_scale: f32,
        restitution: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            velocity,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            linear_damping: 0.02,
            angular_damping: 0.05,
            gravity_scale,
            restitution,
            friction: 0.8,
            ccd: false,
        }
    }

    pub fn with_angular(mut self, angular_velocity: Vec3, angular_damping: f32) -> Self {
        self.angular_velocity = angular_velocity;
        self.angular_damping = angular_damping;
        self
    }

    pub fn with_physical_properties(
        mut self,
        mass: f32,
        linear_damping: f32,
        friction: f32,
        ccd: bool,
    ) -> Self {
        self.mass = mass;
        self.linear_damping = linear_damping;
        self.friction = friction;
        self.ccd = ccd;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxCollider3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub size: Vec3,
    pub offset: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxCollider3dPluginSceneCommandPayload(pub BoxCollider3dSceneCommand);

impl crate::PluginSceneCommandPayload for BoxCollider3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<BoxCollider3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn box_collider_3d_plugin_scene_command(
    command: BoxCollider3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(BoxCollider3dPluginSceneCommandPayload(
        command,
    )))
}

impl BoxCollider3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        size: Vec3,
        offset: Vec3,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            size,
            offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticBoxCollider3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub size: Vec3,
    pub offset: Vec3,
    pub friction: f32,
    pub restitution: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticBoxCollider3dPluginSceneCommandPayload(pub StaticBoxCollider3dSceneCommand);

impl crate::PluginSceneCommandPayload for StaticBoxCollider3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        STATIC_BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<StaticBoxCollider3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn static_box_collider_3d_plugin_scene_command(
    command: StaticBoxCollider3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        StaticBoxCollider3dPluginSceneCommandPayload(command),
    ))
}

impl StaticBoxCollider3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        size: Vec3,
        offset: Vec3,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            size,
            offset,
            friction: 0.9,
            restitution: 0.02,
        }
    }

    pub fn with_surface(mut self, friction: f32, restitution: f32) -> Self {
        self.friction = friction;
        self.restitution = restitution;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsSpawner3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub entity_prefix: String,
    pub mesh: AssetKey,
    pub material: AssetKey,
    pub material_label: String,
    pub interval_seconds: f32,
    pub origin: Vec3,
    pub spawn_scale: Vec3,
    pub grid_spacing: Vec3,
    pub initial_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub spawn_position_jitter: Vec3,
    pub spawn_rotation_jitter: Vec3,
    pub initial_velocity_jitter: Vec3,
    pub angular_velocity_jitter: Vec3,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub restitution: f32,
    pub friction: f32,
    pub ccd: bool,
    pub collider_size: Vec3,
    pub max_alive: u32,
    pub counter_entity: Option<String>,
    pub counter_prefix: String,
    pub counter_font: Option<AssetKey>,
    pub counter_size: f32,
    pub counter_position: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsSpawner3dPluginSceneCommandPayload(pub PhysicsSpawner3dSceneCommand);

impl crate::PluginSceneCommandPayload for PhysicsSpawner3dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        PHYSICS_SPAWNER_3D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<PhysicsSpawner3dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }

    fn asset_dependencies(&self) -> Vec<crate::SceneAssetDependency> {
        let command = &self.0;
        vec![
            crate::SceneAssetDependency::new(
                command.source_mod.clone(),
                command.mesh.clone(),
                "meshes",
                "mesh-3d",
            ),
            crate::SceneAssetDependency::new(
                command.source_mod.clone(),
                command.material.clone(),
                "materials",
                "material-3d",
            ),
        ]
    }
}

pub fn physics_spawner_3d_plugin_scene_command(
    command: PhysicsSpawner3dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        PhysicsSpawner3dPluginSceneCommandPayload(command),
    ))
}
