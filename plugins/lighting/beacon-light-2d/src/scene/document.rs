use std::any::Any;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use amigo_camera::CameraOpticalResponse2dDocument;
use amigo_scene::{
    LayeredImageViewportFit2dDocument, RenderContributionsDocument, RenderDepth2dDocument,
    SceneComponentDocument, SceneComponentPayload, SceneComponentSchemaProvider, SceneDocumentError,
    SceneDocumentResult, SceneVec2Document,
};
use amigo_scene::SceneComponentDocument as ComponentDocument;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BeaconLight2dDocument {
    pub id: String,
    #[serde(default = "default_render_layer")]
    pub render_layer: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_base_intensity")]
    pub base_intensity: f32,
    #[serde(default = "default_frequency_hz")]
    pub frequency_hz: f32,
    #[serde(default = "default_duty_cycle")]
    pub duty_cycle: f32,
    #[serde(default = "default_rise_seconds")]
    pub rise_seconds: f32,
    #[serde(default = "default_fall_seconds")]
    pub fall_seconds: f32,
    #[serde(default)]
    pub phase_offset: f32,
    #[serde(default)]
    pub sync_group: Option<String>,
    #[serde(default = "default_jitter_amount")]
    pub jitter_amount: f32,
    #[serde(default = "default_jitter_hz")]
    pub jitter_hz: f32,
    #[serde(default = "default_core_radius_px")]
    pub core_radius_px: f32,
    #[serde(default = "default_halo_radius_px")]
    pub halo_radius_px: f32,
    #[serde(default = "default_glow_strength")]
    pub glow_strength: f32,
    #[serde(default = "default_true")]
    pub beam_enabled: bool,
    #[serde(default = "default_beam_length_px")]
    pub beam_length_px: f32,
    #[serde(default = "default_beam_width_degrees")]
    pub beam_width_degrees: f32,
    #[serde(default = "default_beam_strength")]
    pub beam_strength: f32,
    #[serde(default = "default_aberration_px")]
    pub aberration_px: f32,
    #[serde(default = "default_bloom")]
    pub bloom: f32,
    #[serde(default)]
    pub camera_response: CameraOpticalResponse2dDocument,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<RenderDepth2dDocument>,
    #[serde(default)]
    pub z_depth: Option<f32>,
    #[serde(default)]
    pub z_index: f32,
    #[serde(default)]
    pub render_contributions: RenderContributionsDocument,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub viewport_fit: LayeredImageViewportFit2dDocument,
    #[serde(default)]
    pub viewport_canvas_size: Option<SceneVec2Document>,
}

impl BeaconLight2dDocument {
    pub fn from_component(component: &SceneComponentDocument) -> Option<Self> {
        match component {
            ComponentDocument::BeaconLight2d {
                id,
                render_layer,
                color,
                base_intensity,
                frequency_hz,
                duty_cycle,
                rise_seconds,
                fall_seconds,
                phase_offset,
                sync_group,
                jitter_amount,
                jitter_hz,
                core_radius_px,
                halo_radius_px,
                glow_strength,
                beam_enabled,
                beam_length_px,
                beam_width_degrees,
                beam_strength,
                aberration_px,
                bloom,
                camera_response,
                depth,
                z_depth,
                z_index,
                render_contributions,
                enabled,
                viewport_fit,
                viewport_canvas_size,
                post_fx: _,
            } => Some(Self {
                id: id.clone(),
                render_layer: render_layer.clone(),
                color: color.clone(),
                base_intensity: *base_intensity,
                frequency_hz: *frequency_hz,
                duty_cycle: *duty_cycle,
                rise_seconds: *rise_seconds,
                fall_seconds: *fall_seconds,
                phase_offset: *phase_offset,
                sync_group: sync_group.clone(),
                jitter_amount: *jitter_amount,
                jitter_hz: *jitter_hz,
                core_radius_px: *core_radius_px,
                halo_radius_px: *halo_radius_px,
                glow_strength: *glow_strength,
                beam_enabled: *beam_enabled,
                beam_length_px: *beam_length_px,
                beam_width_degrees: *beam_width_degrees,
                beam_strength: *beam_strength,
                aberration_px: *aberration_px,
                bloom: *bloom,
                camera_response: *camera_response,
                depth: depth.clone(),
                z_depth: *z_depth,
                z_index: *z_index,
                render_contributions: render_contributions.clone(),
                enabled: *enabled,
                viewport_fit: *viewport_fit,
                viewport_canvas_size: *viewport_canvas_size,
            }),
            _ => None,
        }
    }
}

impl SceneComponentPayload for BeaconLight2dDocument {
    fn component_type(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d.BeaconLight2D"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn parse_beacon_light_2d_plugin_payload(
    payload: &Value,
) -> SceneDocumentResult<BeaconLight2dDocument> {
    serde_yaml::from_value::<BeaconLight2dDocument>(payload.clone())
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

#[derive(Debug, Clone, Copy)]
pub struct BeaconLight2dSceneSchemaProvider;

impl SceneComponentSchemaProvider for BeaconLight2dSceneSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d.BeaconLight2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["BeaconLight2D"]
    }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<BeaconLight2dDocument>(
            Value::Mapping(payload),
        )?)
    }

    fn parse_payload_value(&self, payload: &Value) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        Ok(Box::new(parse_beacon_light_2d_plugin_payload(payload)?))
    }
}

fn default_render_layer() -> String {
    "default".to_owned()
}

fn default_color() -> String {
    "#FFFFFFFF".to_owned()
}

fn default_base_intensity() -> f32 { 1.0 }
fn default_frequency_hz() -> f32 { 1.0 }
fn default_duty_cycle() -> f32 { 0.2 }
fn default_rise_seconds() -> f32 { 0.1 }
fn default_fall_seconds() -> f32 { 0.2 }
fn default_jitter_amount() -> f32 { 0.06 }
fn default_jitter_hz() -> f32 { 9.0 }
fn default_core_radius_px() -> f32 { 2.0 }
fn default_halo_radius_px() -> f32 { 9.0 }
fn default_glow_strength() -> f32 { 1.0 }
fn default_beam_length_px() -> f32 { 0.0 }
fn default_beam_width_degrees() -> f32 { 20.0 }
fn default_beam_strength() -> f32 { 0.0 }
fn default_aberration_px() -> f32 { 0.8 }
fn default_bloom() -> f32 { 1.0 }
fn default_true() -> bool { true }
