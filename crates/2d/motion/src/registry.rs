use std::collections::BTreeMap;

use crate::{
    bounds::Bounds2dCommand,
    controller::{MotionController2dCommand, MotionIntent2d, MotionState2d},
    freeflight::{FreeflightMotion2dCommand, FreeflightMotionIntent2d, FreeflightMotionState2d},
    projectile::ProjectileEmitter2dCommand,
    velocity::Velocity2dCommand,
};

#[derive(Debug, Default)]
pub(crate) struct MotionStateRegistry {
    pub(crate) commands: BTreeMap<String, MotionController2dCommand>,
    pub(crate) states: BTreeMap<String, MotionState2d>,
    pub(crate) motors: BTreeMap<String, MotionIntent2d>,
    pub(crate) velocities: BTreeMap<String, Velocity2dCommand>,
    pub(crate) bounds: BTreeMap<String, Bounds2dCommand>,
    pub(crate) freeflight_commands: BTreeMap<String, FreeflightMotion2dCommand>,
    pub(crate) freeflight_states: BTreeMap<String, FreeflightMotionState2d>,
    pub(crate) freeflight_intents: BTreeMap<String, FreeflightMotionIntent2d>,
    pub(crate) projectile_emitters: BTreeMap<String, ProjectileEmitter2dCommand>,
}
