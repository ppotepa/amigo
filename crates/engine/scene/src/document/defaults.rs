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

pub(super) fn default_light3d_direction() -> SceneVec3Document {
    SceneVec3Document {
        x: -0.35,
        y: -0.8,
        z: -0.45,
    }
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

pub(super) fn default_lightmap_sample_points() -> u32 {
    5
}

pub(super) fn default_lightmap_sample_radius_px() -> f32 {
    16.0
}

pub(super) fn default_lightmap_exposure() -> f32 {
    1.0
}

pub(super) fn default_light_receiver_global_response() -> f32 {
    1.0
}

pub(super) fn default_gravity_scale() -> f32 {
    1.0
}

pub(super) fn default_physics3d_gravity() -> SceneVec3Document {
    SceneVec3Document {
        x: 0.0,
        y: -9.81,
        z: 0.0,
    }
}

pub(super) fn default_physics3d_substeps() -> u32 {
    1
}

pub(super) fn default_physics3d_solver_iterations() -> u32 {
    4
}

pub(super) fn default_physics3d_ccd_substeps() -> u32 {
    1
}

pub(super) fn default_camera3d_fov_y_degrees() -> f32 {
    55.0
}

pub(super) fn default_camera3d_near_clip() -> f32 {
    0.1
}

pub(super) fn default_camera3d_far_clip() -> f32 {
    100.0
}

pub(super) fn default_light3d_intensity() -> f32 {
    0.85
}

pub(super) fn default_light3d_ambient() -> f32 {
    0.25
}

pub(super) fn default_rigid_body_mass_3d() -> f32 {
    1.0
}

pub(super) fn default_linear_damping_3d() -> f32 {
    0.02
}

pub(super) fn default_spawn_interval_seconds() -> f32 {
    1.0
}

pub(super) fn default_physics3d_counter_prefix() -> String {
    "CUBES: ".to_owned()
}

pub(super) fn default_physics3d_counter_size() -> f32 {
    0.28
}

pub(super) fn default_angular_damping_3d() -> f32 {
    0.05
}

pub(super) fn default_rigid_body_friction_3d() -> f32 {
    0.8
}

pub(super) fn default_vector_segments() -> u32 {
    16
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
