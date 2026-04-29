use std::sync::Arc;

use amigo_2d_platformer::{PlatformerFacing, PlatformerSceneService};

#[derive(Clone)]
pub struct PlatformerApi {
    pub(crate) platformer_scene: Option<Arc<PlatformerSceneService>>,
}

pub type MotionApi = PlatformerApi;

#[derive(Clone, Default)]
pub struct PlatformerStateView {
    grounded: bool,
    facing: String,
    animation: String,
    velocity_x: rhai::FLOAT,
    velocity_y: rhai::FLOAT,
}

pub type MotionStateView = PlatformerStateView;

impl PlatformerStateView {
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
}

impl PlatformerApi {
    pub fn drive(
        &mut self,
        entity_name: &str,
        move_x: rhai::FLOAT,
        jump_pressed: bool,
        jump_held: bool,
        _delta_seconds: rhai::FLOAT,
    ) -> bool {
        self.platformer_scene
            .as_ref()
            .map(|scene| scene.drive(entity_name, move_x as f32, jump_pressed, jump_held))
            .unwrap_or(false)
    }

    pub fn state(&mut self, entity_name: &str) -> PlatformerStateView {
        let Some(platformer_scene) = self.platformer_scene.as_ref() else {
            return PlatformerStateView::default();
        };
        let Some(state) = platformer_scene.state(entity_name) else {
            return PlatformerStateView::default();
        };

        PlatformerStateView {
            grounded: state.grounded,
            facing: match state.facing {
                PlatformerFacing::Left => "left".to_owned(),
                PlatformerFacing::Right => "right".to_owned(),
            },
            animation: format!("{:?}", state.animation).to_ascii_lowercase(),
            velocity_x: state.velocity.x as rhai::FLOAT,
            velocity_y: state.velocity.y as rhai::FLOAT,
        }
    }
}
