use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds2d {
    pub min: Vec2,
    pub max: Vec2,
    pub behavior: BoundsBehavior2d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsBehavior2d {
    Bounce { restitution: f32 },
    Wrap,
    Hide,
    Despawn,
    Clamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundsContact2d {
    pub min_x: bool,
    pub max_x: bool,
    pub min_y: bool,
    pub max_y: bool,
}

impl BoundsContact2d {
    pub const fn any(self) -> bool {
        self.min_x || self.max_x || self.min_y || self.max_y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsOutcome2d {
    None,
    Bounced { contact: BoundsContact2d },
    Wrapped { contact: BoundsContact2d },
    Clamped { contact: BoundsContact2d },
    Hidden { contact: BoundsContact2d },
    Despawned { contact: BoundsContact2d },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundsApplyResult2d {
    pub translation: Vec2,
    pub velocity: Vec2,
    pub outcome: BoundsOutcome2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bounds2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub bounds: Bounds2d,
}

pub fn apply_bounds_2d(
    translation: Vec2,
    velocity: Vec2,
    bounds: &Bounds2d,
) -> BoundsApplyResult2d {
    let contact = bounds_contact_2d(translation, bounds);
    if !contact.any() {
        return BoundsApplyResult2d {
            translation,
            velocity,
            outcome: BoundsOutcome2d::None,
        };
    }

    match bounds.behavior {
        BoundsBehavior2d::Bounce { restitution } => {
            let restitution = restitution.max(0.0);
            let mut next_translation = clamp_to_bounds_2d(translation, bounds);
            let mut next_velocity = velocity;
            if contact.min_x {
                next_translation.x = bounds.min.x;
                next_velocity.x = velocity.x.abs() * restitution;
            } else if contact.max_x {
                next_translation.x = bounds.max.x;
                next_velocity.x = -velocity.x.abs() * restitution;
            }
            if contact.min_y {
                next_translation.y = bounds.min.y;
                next_velocity.y = velocity.y.abs() * restitution;
            } else if contact.max_y {
                next_translation.y = bounds.max.y;
                next_velocity.y = -velocity.y.abs() * restitution;
            }
            BoundsApplyResult2d {
                translation: next_translation,
                velocity: next_velocity,
                outcome: BoundsOutcome2d::Bounced { contact },
            }
        }
        BoundsBehavior2d::Wrap => {
            let mut next_translation = translation;
            if contact.min_x {
                next_translation.x = bounds.max.x;
            } else if contact.max_x {
                next_translation.x = bounds.min.x;
            }
            if contact.min_y {
                next_translation.y = bounds.max.y;
            } else if contact.max_y {
                next_translation.y = bounds.min.y;
            }
            BoundsApplyResult2d {
                translation: next_translation,
                velocity,
                outcome: BoundsOutcome2d::Wrapped { contact },
            }
        }
        BoundsBehavior2d::Hide => BoundsApplyResult2d {
            translation,
            velocity,
            outcome: BoundsOutcome2d::Hidden { contact },
        },
        BoundsBehavior2d::Despawn => BoundsApplyResult2d {
            translation,
            velocity,
            outcome: BoundsOutcome2d::Despawned { contact },
        },
        BoundsBehavior2d::Clamp => BoundsApplyResult2d {
            translation: clamp_to_bounds_2d(translation, bounds),
            velocity,
            outcome: BoundsOutcome2d::Clamped { contact },
        },
    }
}

fn bounds_contact_2d(translation: Vec2, bounds: &Bounds2d) -> BoundsContact2d {
    BoundsContact2d {
        min_x: translation.x < bounds.min.x,
        max_x: translation.x > bounds.max.x,
        min_y: translation.y < bounds.min.y,
        max_y: translation.y > bounds.max.y,
    }
}

fn clamp_to_bounds_2d(translation: Vec2, bounds: &Bounds2d) -> Vec2 {
    Vec2::new(
        translation.x.clamp(bounds.min.x, bounds.max.x),
        translation.y.clamp(bounds.min.y, bounds.max.y),
    )
}
