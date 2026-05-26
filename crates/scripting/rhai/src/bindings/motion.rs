use std::sync::Arc;

use amigo_shutter_motion_plugin::{
    Facing2d, FreeflightMotionIntent2d, Motion2dSceneService, motion_facing_to_str,
};
use rhai::{Dynamic, FLOAT, INT, Map};

#[derive(Clone)]
pub struct MotionApi {
    pub(crate) motion_scene: Option<Arc<Motion2dSceneService>>,
}

#[derive(Clone, Default)]
pub struct MotionStateView {
    grounded: bool,
    facing: String,
    animation: String,
    velocity_x: rhai::FLOAT,
    velocity_y: rhai::FLOAT,
    angle_radians: rhai::FLOAT,
}

impl MotionStateView {
    pub fn grounded(&mut self) -> bool {
        self.grounded
    }

    pub fn facing(&mut self) -> String {
        self.facing.clone()
    }

    pub fn animation(&mut self) -> String {
        self.animation.clone()
    }

    pub fn velocity_x(&mut self) -> rhai::FLOAT {
        self.velocity_x
    }

    pub fn velocity_y(&mut self) -> rhai::FLOAT {
        self.velocity_y
    }

    pub fn velocity_x_int(&mut self) -> rhai::INT {
        self.velocity_x.round() as rhai::INT
    }

    pub fn velocity_y_int(&mut self) -> rhai::INT {
        self.velocity_y.round() as rhai::INT
    }

    pub fn angle_radians(&mut self) -> rhai::FLOAT {
        self.angle_radians
    }
}

impl MotionApi {
    pub fn drive(
        &mut self,
        entity_name: &str,
        move_x: rhai::FLOAT,
        jump_pressed: bool,
        jump_held: bool,
        _delta_seconds: rhai::FLOAT,
    ) -> bool {
        self.motion_scene
            .as_ref()
            .map(|scene| scene.drive_motion(entity_name, move_x as f32, jump_pressed, jump_held))
            .unwrap_or(false)
    }

    pub fn drive_freeflight(&mut self, entity_name: &str, intent: Map) -> bool {
        let Some(motion_scene) = self.motion_scene.as_ref() else {
            return false;
        };

        motion_scene.drive_freeflight(
            entity_name,
            FreeflightMotionIntent2d {
                thrust: map_number(&intent, "thrust").unwrap_or(0.0),
                strafe: map_number(&intent, "strafe").unwrap_or(0.0),
                turn: map_number(&intent, "turn").unwrap_or(0.0),
            },
        )
    }

    pub fn set_velocity(&mut self, entity_name: &str, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        self.motion_scene
            .as_ref()
            .map(|scene| scene.set_velocity(entity_name, amigo_math::Vec2::new(x as f32, y as f32)))
            .unwrap_or(false)
    }

    pub fn reset_freeflight(&mut self, entity_name: &str, rotation: rhai::FLOAT) -> bool {
        self.motion_scene
            .as_ref()
            .map(|scene| scene.reset_freeflight(entity_name, rotation as f32))
            .unwrap_or(false)
    }

    pub fn state(&mut self, entity_name: &str) -> MotionStateView {
        let Some(motion_scene) = self.motion_scene.as_ref() else {
            return MotionStateView::default();
        };
        let Some(state) = motion_scene.motion_state(entity_name) else {
            let Some(state) = motion_scene.freeflight_state(entity_name) else {
                return MotionStateView::default();
            };

            return MotionStateView {
                grounded: false,
                facing: "right".to_owned(),
                animation: "freeflight".to_owned(),
                velocity_x: state.velocity.x as rhai::FLOAT,
                velocity_y: state.velocity.y as rhai::FLOAT,
                angle_radians: state.rotation_radians as rhai::FLOAT,
            };
        };

        MotionStateView {
            grounded: state.grounded,
            facing: motion_facing_to_str(match state.facing {
                Facing2d::Left => Facing2d::Left,
                Facing2d::Right => Facing2d::Right,
            })
            .to_owned(),
            animation: format!("{:?}", state.animation).to_ascii_lowercase(),
            velocity_x: state.velocity.x as rhai::FLOAT,
            velocity_y: state.velocity.y as rhai::FLOAT,
            angle_radians: 0.0,
        }
    }
}

fn map_number(map: &Map, key: &str) -> Option<f32> {
    let value = map.get(key)?;
    dynamic_to_f32(value.clone())
}

fn dynamic_to_f32(value: Dynamic) -> Option<f32> {
    if let Some(number) = value.clone().try_cast::<FLOAT>() {
        return Some(number as f32);
    }
    if let Some(number) = value.try_cast::<INT>() {
        return Some(number as f32);
    }
    None
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<MotionApi>("WorldMotion")
        .register_type_with_name::<MotionStateView>("MotionState")
        .register_fn("drive", MotionApi::drive)
        .register_fn("drive_freeflight", MotionApi::drive_freeflight)
        .register_fn("set_velocity", MotionApi::set_velocity)
        .register_fn("reset_freeflight", MotionApi::reset_freeflight)
        .register_fn("state", MotionApi::state)
        .register_get("grounded", MotionStateView::grounded)
        .register_get("facing", MotionStateView::facing)
        .register_get("animation", MotionStateView::animation)
        .register_get("velocity_x", MotionStateView::velocity_x)
        .register_get("velocity_y", MotionStateView::velocity_y)
        .register_get("velocity_x_int", MotionStateView::velocity_x_int)
        .register_get("velocity_y_int", MotionStateView::velocity_y_int)
        .register_get("angle_radians", MotionStateView::angle_radians);
}
