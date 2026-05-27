use amigo_math::Vec3;
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsWorld3d {
    pub gravity: Vec3,
    pub substeps: u32,
    pub solver_iterations: u32,
    pub ccd_substeps: u32,
}

impl Default for PhysicsWorld3d {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            substeps: 1,
            solver_iterations: 4,
            ccd_substeps: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigidBody3d {
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

impl Default for RigidBody3d {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            linear_damping: 0.02,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            restitution: 0.0,
            friction: 0.8,
            ccd: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsBodyState3d {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub grounded: bool,
}

impl Default for PhysicsBodyState3d {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            grounded: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsSpawner3d {
    pub entity_prefix: String,
    pub mesh: String,
    pub material: String,
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
    pub counter_font: String,
    pub counter_size: f32,
    pub counter_position: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsSpawner3dState {
    pub elapsed_seconds: f32,
    pub spawn_index: u32,
}

impl Default for PhysicsSpawner3dState {
    fn default() -> Self {
        Self {
            elapsed_seconds: 0.0,
            spawn_index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxCollider3d {
    pub size: Vec3,
    pub offset: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticBoxCollider3d {
    pub size: Vec3,
    pub offset: Vec3,
    pub friction: f32,
    pub restitution: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigidBody3dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub body: RigidBody3d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsSpawner3dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub source_mod: String,
    pub spawner: PhysicsSpawner3d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxCollider3dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub collider: BoxCollider3d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticBoxCollider3dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub collider: StaticBoxCollider3d,
}
