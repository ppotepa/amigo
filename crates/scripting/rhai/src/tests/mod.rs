use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use amigo_2d_motion::{
    Facing2d, FreeflightMotion2dCommand, FreeflightMotionProfile2d, FreeflightMotionState2d,
    Motion2dSceneService, MotionAnimationState, MotionController2d, MotionController2dCommand,
    MotionIntent2d, MotionProfile2d, MotionState2d, ProjectileEmitter2d,
    ProjectileEmitter2dCommand,
};
use amigo_2d_particles::{
    Particle2dSceneService, ParticleEmitter2d, ParticleEmitter2dCommand, ParticleShape2d,
};
use amigo_2d_physics::{CircleCollider2d, CircleCollider2dCommand, Physics2dSceneService};
use amigo_2d_sprite::{Sprite, SpriteDrawCommand, SpriteSceneService, SpriteSheet};
use amigo_2d_vector::{
    VectorSceneService, VectorShape2d, VectorShape2dDrawCommand, VectorShapeKind2d, VectorStyle2d,
};
use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
    PreparedAssetKind, prepare_debug_placeholder_asset,
};
use amigo_core::{LaunchSelection, RuntimeDiagnostics};
use amigo_input_actions::{InputActionBinding, InputActionId, InputActionMap, InputActionService};
use amigo_input_api::{InputState, KeyCode};
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
use amigo_modding::{DiscoveredMod, ModCatalog, ModManifest, ModSceneManifest};
use amigo_scene::{
    EntityPoolSceneCommand, EntityPoolSceneService, SceneEntityId, SceneEntityLifecycle, SceneKey,
    ScenePropertyValue, SceneService,
};
use amigo_scripting_api::{
    DevConsoleQueue, ScriptCommand, ScriptCommandQueue, ScriptEventQueue, ScriptTraceService,
    ScriptValue,
};
use amigo_state::{SceneStateService, SceneTimerService, SessionStateService};
use amigo_ui::{UiTheme, UiThemePalette, UiThemeService};

use crate::RhaiScriptRuntime;
use amigo_scripting_api::ScriptRuntime;

fn test_freeflight_profile() -> FreeflightMotionProfile2d {
    FreeflightMotionProfile2d {
        thrust_acceleration: 1.0,
        reverse_acceleration: 1.0,
        strafe_acceleration: 0.0,
        turn_acceleration: 1.0,
        linear_damping: 0.0,
        turn_damping: 0.0,
        max_speed: 10.0,
        max_angular_speed: 10.0,
        thrust_response_curve: amigo_math::Curve1d::Linear,
        reverse_response_curve: amigo_math::Curve1d::Linear,
        strafe_response_curve: amigo_math::Curve1d::Linear,
        turn_response_curve: amigo_math::Curve1d::Linear,
    }
}

fn test_particle_emitter() -> ParticleEmitter2d {
    ParticleEmitter2d {
        attached_to: None,
        local_offset: Vec2::ZERO,
        local_direction_radians: 0.0,
        spawn_area: amigo_2d_particles::ParticleSpawnArea2d::Point,
        active: false,
        spawn_rate: 10.0,
        max_particles: 16,
        particle_lifetime: 1.0,
        lifetime_jitter: 0.0,
        initial_speed: 0.0,
        speed_jitter: 0.0,
        spread_radians: 0.0,
        inherit_parent_velocity: 0.0,
        velocity_mode: amigo_2d_particles::ParticleVelocityMode2d::Free,
        simulation_space: amigo_2d_particles::ParticleSimulationSpace2d::World,
        initial_size: 1.0,
        final_size: 1.0,
        color: ColorRgba::WHITE,
        color_ramp: None,
        z_index: 1.0,
        shape: ParticleShape2d::Circle { segments: 8 },
        shape_choices: Vec::new(),
        shape_over_lifetime: Vec::new(),
        line_anchor: amigo_2d_particles::ParticleLineAnchor2d::Center,
        align: amigo_2d_particles::ParticleAlignMode2d::Velocity,
        blend_mode: amigo_2d_particles::ParticleBlendMode2d::Alpha,
        motion_stretch: None,
        material: amigo_2d_particles::ParticleMaterial2d {
            receives_light: false,
            light_response: 1.0,
        },
        light: None,
        emission_rate_curve: amigo_math::Curve1d::Constant(1.0),
        size_curve: amigo_math::Curve1d::Constant(1.0),
        alpha_curve: amigo_math::Curve1d::Constant(1.0),
        speed_curve: amigo_math::Curve1d::Constant(1.0),
        forces: Vec::new(),
    }
}

mod component_lifecycle_tests;
mod import_resolver_tests;
mod lifecycle_source_tests;
mod package_tests;
mod world_api_queries;
mod world_api_motion;
mod world_api_lifecycle;

fn discovered_mod(id: &str, capabilities: &[&str], scenes: &[&str]) -> DiscoveredMod {
    DiscoveredMod {
        manifest: ModManifest {
            id: id.to_owned(),
            name: id.to_owned(),
            version: "0.1.0".to_owned(),
            description: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            capabilities: capabilities
                .iter()
                .map(|capability| (*capability).to_owned())
                .collect(),
            scripting: None,
            scenes: scenes
                .iter()
                .map(|scene_id| ModSceneManifest {
                    id: (*scene_id).to_owned(),
                    label: scene_id.to_string(),
                    description: None,
                    path: format!("scenes/{scene_id}"),
                    document: None,
                    script: None,
                    launcher_visible: true,
                })
                .collect(),
        },
        root_path: PathBuf::from(format!("mods/{id}")),
    }
}
