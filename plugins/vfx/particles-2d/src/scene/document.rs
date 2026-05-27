use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_scene::{
    ColorRampSceneDocument, Curve1dSceneDocument, ParticleAlignMode2dSceneDocument,
    ParticleBlendMode2dSceneDocument, ParticleForce2dSceneDocument, ParticleLight2dSceneDocument,
    ParticleLineAnchor2dSceneDocument, ParticleMaterial2dSceneDocument,
    ParticleMotionStretch2dSceneDocument, ParticleShape2dSceneDocument,
    ParticleShapeChoice2dSceneDocument, ParticleShapeKeyframe2dSceneDocument,
    ParticleSimulationSpace2dSceneDocument, ParticleSpawnArea2dSceneDocument,
    ParticleVelocityMode2dSceneDocument, PostFx2dDocument, SceneComponentDocument,
    SceneComponentPayload, SceneComponentSchemaProvider, SceneDocumentError, SceneDocumentResult,
    SceneVec2Document,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParticleEmitter2dDocument {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    #[serde(default)]
    pub attached_to: Option<String>,
    #[serde(default = "default_vec2_zero")]
    pub local_offset: SceneVec2Document,
    #[serde(default)]
    pub local_direction_degrees: f32,
    #[serde(default)]
    pub spawn_area: Option<ParticleSpawnArea2dSceneDocument>,
    #[serde(default)]
    pub active: bool,
    #[serde(default = "default_particle_spawn_rate")]
    pub spawn_rate: f32,
    #[serde(default = "default_particle_max_particles")]
    pub max_particles: usize,
    #[serde(default = "default_particle_lifetime")]
    pub particle_lifetime: f32,
    #[serde(default)]
    pub lifetime_jitter: f32,
    #[serde(default)]
    pub initial_speed: f32,
    #[serde(default)]
    pub speed_jitter: f32,
    #[serde(default)]
    pub spread_degrees: f32,
    #[serde(default)]
    pub inherit_parent_velocity: f32,
    #[serde(default)]
    pub velocity_mode: Option<ParticleVelocityMode2dSceneDocument>,
    #[serde(default)]
    pub simulation_space: Option<ParticleSimulationSpace2dSceneDocument>,
    #[serde(default = "default_particle_initial_size")]
    pub initial_size: f32,
    #[serde(default = "default_particle_final_size")]
    pub final_size: f32,
    #[serde(default)]
    pub size_jitter: f32,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub color_ramp: Option<ColorRampSceneDocument>,
    #[serde(default)]
    pub z_index: f32,
    #[serde(default)]
    pub shape: Option<ParticleShape2dSceneDocument>,
    #[serde(default)]
    pub shape_choices: Vec<ParticleShapeChoice2dSceneDocument>,
    #[serde(default)]
    pub shape_over_lifetime: Vec<ParticleShapeKeyframe2dSceneDocument>,
    #[serde(default)]
    pub line_anchor: Option<ParticleLineAnchor2dSceneDocument>,
    #[serde(default)]
    pub align: Option<ParticleAlignMode2dSceneDocument>,
    #[serde(default)]
    pub blend_mode: Option<ParticleBlendMode2dSceneDocument>,
    #[serde(default)]
    pub motion_stretch: Option<ParticleMotionStretch2dSceneDocument>,
    #[serde(default)]
    pub material: Option<ParticleMaterial2dSceneDocument>,
    #[serde(default)]
    pub light: Option<ParticleLight2dSceneDocument>,
    #[serde(default)]
    pub emission_rate_curve: Option<Curve1dSceneDocument>,
    #[serde(default)]
    pub size_curve: Option<Curve1dSceneDocument>,
    #[serde(default)]
    pub alpha_curve: Option<Curve1dSceneDocument>,
    #[serde(default)]
    pub speed_curve: Option<Curve1dSceneDocument>,
    #[serde(default)]
    pub forces: Vec<ParticleForce2dSceneDocument>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_fx: Vec<PostFx2dDocument>,
}

impl Default for ParticleEmitter2dDocument {
    fn default() -> Self {
        Self {
            entity_name: String::new(),
            render_layer: default_render_layer(),
            attached_to: None,
            local_offset: default_vec2_zero(),
            local_direction_degrees: 0.0,
            spawn_area: None,
            active: false,
            spawn_rate: default_particle_spawn_rate(),
            max_particles: default_particle_max_particles(),
            particle_lifetime: default_particle_lifetime(),
            lifetime_jitter: 0.0,
            initial_speed: 0.0,
            speed_jitter: 0.0,
            spread_degrees: 0.0,
            inherit_parent_velocity: 0.0,
            velocity_mode: None,
            simulation_space: None,
            initial_size: default_particle_initial_size(),
            final_size: default_particle_final_size(),
            size_jitter: 0.0,
            color: None,
            color_ramp: None,
            z_index: 0.0,
            shape: None,
            shape_choices: Vec::new(),
            shape_over_lifetime: Vec::new(),
            line_anchor: None,
            align: None,
            blend_mode: None,
            motion_stretch: None,
            material: None,
            light: None,
            emission_rate_curve: None,
            size_curve: None,
            alpha_curve: None,
            speed_curve: None,
            forces: Vec::new(),
            post_fx: Vec::new(),
        }
    }
}

impl ParticleEmitter2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.vfx.particles-2d.ParticleEmitter2D"
                || component_type == "ParticleEmitter2D" =>
            {
                parse_particle_emitter_2d_plugin_payload(payload).ok()
            }
            _ => None,
        }
    }
}

impl SceneComponentPayload for ParticleEmitter2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.vfx.particles-2d.ParticleEmitter2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_particle_emitter_2d_plugin_payload(
    payload: &Value,
) -> SceneDocumentResult<ParticleEmitter2dDocument> {
    serde_yaml::from_value::<ParticleEmitter2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ParticleEmitter2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for ParticleEmitter2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.vfx.particles-2d.ParticleEmitter2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["ParticleEmitter2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<ParticleEmitter2dDocument>(
            Value::Mapping(payload),
        )?)
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_particle_emitter_2d_plugin_payload(payload)?))
    }
}

fn default_render_layer() -> String {
    "default".to_owned()
}

fn default_vec2_zero() -> SceneVec2Document {
    SceneVec2Document { x: 0.0, y: 0.0 }
}

fn default_particle_spawn_rate() -> f32 {
    10.0
}

fn default_particle_max_particles() -> usize {
    128
}

fn default_particle_lifetime() -> f32 {
    1.0
}

fn default_particle_initial_size() -> f32 {
    1.0
}

fn default_particle_final_size() -> f32 {
    1.0
}
