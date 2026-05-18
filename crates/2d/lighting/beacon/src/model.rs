use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_render_api::{
    render_contribution_roles as roles, RenderContributionSet,
};
use amigo_scene::{BeaconLight2dSceneCommand, LayeredImageViewportFit2dSceneCommand};

pub const BEACON_2D_CAPABILITY: &str = "beacon_2d";
pub const BEACON_2D_PLUGIN_LABEL: &str = "amigo-2d-lighting-beacon";

#[derive(Debug, Clone, PartialEq)]
pub struct BeaconLight2dCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub render_layer: String,
    pub color: ColorRgba,
    pub base_intensity: f32,
    pub frequency_hz: f32,
    pub duty_cycle: f32,
    pub rise_seconds: f32,
    pub fall_seconds: f32,
    pub phase_offset: f32,
    pub sync_group: Option<String>,
    pub jitter_amount: f32,
    pub jitter_hz: f32,
    pub core_radius_px: f32,
    pub halo_radius_px: f32,
    pub glow_strength: f32,
    pub beam_enabled: bool,
    pub beam_length_px: f32,
    pub beam_width_degrees: f32,
    pub beam_strength: f32,
    pub aberration_px: f32,
    pub flare_length_px: f32,
    pub flare_strength: f32,
    pub bloom: f32,
    pub lens_influence: f32,
    pub distance_m: Option<f32>,
    pub z_depth: Option<f32>,
    pub z_index: f32,
    pub render_contributions: RenderContributionSet,
    pub enabled: bool,
    pub transform: Transform2,
    pub viewport_fit: LayeredImageViewportFit2dSceneCommand,
    pub viewport_canvas_size: Option<Vec2>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeaconLight2dDrawCommand {
    pub entity_name: String,
    pub render_layer: String,
    pub z_index: f32,
    pub center: Vec2,
    pub color: ColorRgba,
    pub intensity: f32,
    pub pulse: f32,
    pub core_radius_px: f32,
    pub halo_radius_px: f32,
    pub glow_strength: f32,
    pub rotation_radians: f32,
    pub beam_enabled: bool,
    pub beam_length_px: f32,
    pub beam_width_degrees: f32,
    pub beam_strength: f32,
    pub aberration_px: f32,
    pub flare_length_px: f32,
    pub flare_strength: f32,
    pub bloom: f32,
    pub lens_influence: f32,
    pub distance_m: Option<f32>,
    pub z_depth: Option<f32>,
    pub render_contributions: RenderContributionSet,
    pub viewport_fit: LayeredImageViewportFit2dSceneCommand,
    pub viewport_canvas_size: Option<Vec2>,
}

impl From<&BeaconLight2dSceneCommand> for BeaconLight2dCommand {
    fn from(value: &BeaconLight2dSceneCommand) -> Self {
        let mut render_contributions =
            RenderContributionSet::from_pairs(value.render_contributions.roles.clone());
        render_contributions.merge_defaults([
            (roles::OVERLAY_VISIBLE, true),
            (roles::RELIGHT_PLATE, true),
            (roles::BLOOM_SOURCE, true),
            (roles::CAMERA_FX_SOURCE, true),
        ]);

        Self {
            source_mod: value.source_mod.clone(),
            entity_name: value.entity_name.clone(),
            id: value.id.clone(),
            render_layer: value.render_layer.clone(),
            color: value.color,
            base_intensity: value.base_intensity,
            frequency_hz: value.frequency_hz,
            duty_cycle: value.duty_cycle,
            rise_seconds: value.rise_seconds,
            fall_seconds: value.fall_seconds,
            phase_offset: value.phase_offset,
            sync_group: value.sync_group.clone(),
            jitter_amount: value.jitter_amount,
            jitter_hz: value.jitter_hz,
            core_radius_px: value.core_radius_px,
            halo_radius_px: value.halo_radius_px,
            glow_strength: value.glow_strength,
            beam_enabled: value.beam_enabled,
            beam_length_px: value.beam_length_px,
            beam_width_degrees: value.beam_width_degrees,
            beam_strength: value.beam_strength,
            aberration_px: value.aberration_px,
            flare_length_px: value.flare_length_px,
            flare_strength: value.flare_strength,
            bloom: value.bloom,
            lens_influence: value.lens_influence,
            distance_m: value.depth.as_ref().and_then(|depth| depth.distance_m),
            z_depth: value.z_depth.map(|z_depth| z_depth.clamp(0.0, 1.0)),
            z_index: value.z_index,
            render_contributions,
            enabled: value.enabled,
            transform: value.transform,
            viewport_fit: value.viewport_fit,
            viewport_canvas_size: value.viewport_canvas_size,
        }
    }
}
