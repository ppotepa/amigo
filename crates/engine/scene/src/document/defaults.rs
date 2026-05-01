use super::particles::ParticleLightMode2dSceneDocument;
use super::render_values::{SceneVec2Document, SceneVec3Document};

pub(super) fn default_scene_document_version() -> u32 {
    1
}

pub(super) fn default_vec2_zero() -> SceneVec2Document {
    SceneVec2Document::ZERO
}

pub(super) fn default_vec2_one() -> SceneVec2Document {
    SceneVec2Document::ONE
}

pub(super) fn default_vec3_zero() -> SceneVec3Document {
    SceneVec3Document::ZERO
}

pub(super) fn default_vec3_one() -> SceneVec3Document {
    SceneVec3Document::ONE
}

pub(super) fn default_sprite_sheet_fps() -> f32 {
    8.0
}

pub(super) fn default_sprite_sheet_looping() -> bool {
    true
}

pub(super) fn default_gravity_scale() -> f32 {
    1.0
}

pub(super) fn default_vector_segments() -> u32 {
    16
}

pub(super) fn default_vector_stroke_width() -> f32 {
    1.0
}

pub(super) fn default_particle_spawn_rate() -> f32 {
    10.0
}

pub(super) fn default_particle_max_particles() -> usize {
    128
}

pub(super) fn default_particle_lifetime() -> f32 {
    1.0
}

pub(super) fn default_particle_initial_size() -> f32 {
    1.0
}

pub(super) fn default_particle_final_size() -> f32 {
    1.0
}

pub(super) fn default_particle_shape_choice_weight() -> f32 {
    1.0
}

pub(super) fn default_particle_light_response() -> f32 {
    1.0
}

impl Default for ParticleLightMode2dSceneDocument {
    fn default() -> Self {
        Self::Source
    }
}

pub(super) fn default_ui_font_size() -> f32 {
    16.0
}

pub(super) fn default_camera_follow_lerp() -> f32 {
    1.0
}

pub(super) fn default_bounds_restitution() -> f32 {
    1.0
}

pub(super) fn default_entity_lifecycle_flag() -> bool {
    true
}

pub(super) fn default_once_per_overlap() -> bool {
    true
}
