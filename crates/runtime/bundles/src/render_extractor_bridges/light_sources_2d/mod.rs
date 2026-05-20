use amigo_camera_optics_plugin::api::{
    CameraOpticalCandidate2d, CameraOpticalEmitterKind2d, CameraOpticalResponse2d,
    CameraOpticalSource2d, CameraOpticalSourceStatus2d,
};
use amigo_material_2d_plugin::Material2d;
use amigo_render_api::{
    CameraCaptureInput2d, LightContributionKind2d, LightEmitterKind2d, LightSource2dCommon,
    LightSource2dCommonParams, LightSourceStatus2d, RenderContributionSet,
    VisualSourceAvailability2d,
};

use crate::render_extractor_bridges::visual_2d_items::{Renderable2dItem, Renderable2dPayload};

const MAX_BEACON_LIGHT_SOURCES: usize = 32;
const MAX_GLOBAL_LIGHT_SOURCES: usize = 16;
const MAX_LIGHTMAP_SOURCES: usize = 16;
const MAX_LIGHT_GROUP_SOURCES: usize = 16;
const MAX_PARTICLE_LIGHT_SOURCES: usize = 64;
const MAX_MATERIAL_EMISSIVE_LIGHT_SOURCES: usize = 64;

macro_rules! light_source_params {
    (
        $owner:expr,
        $component_kind:expr,
        $emitter_kind:expr,
        $emitter_id:expr,
        $render_layer:expr,
        $color_rgba:expr,
        $intensity:expr,
        $effective_intensity:expr,
        $response:expr,
        $camera_response:expr,
        $bloom:expr,
        $radius_px:expr,
        $falloff:expr,
        $distance_m:expr,
        $z_depth:expr,
        $contributions:expr,
        $reason:expr,
        $position_px:expr $(,)?
    ) => {
        LightSource2dCommonParams {
            owner: $owner.into(),
            component_kind: $component_kind.into(),
            emitter_kind: $emitter_kind,
            emitter_id: $emitter_id,
            render_layer: $render_layer,
            color_rgba: $color_rgba,
            intensity: $intensity,
            effective_intensity: $effective_intensity,
            response: $response,
            camera_response: $camera_response,
            bloom: $bloom,
            radius_px: $radius_px,
            falloff: $falloff,
            distance_m: $distance_m,
            z_depth: $z_depth,
            contributions: $contributions,
            reason: $reason.into(),
            position_px: $position_px,
        }
    };
}

macro_rules! active_light_source {
    ($($params:tt)*) => {
        LightSource2dCommon::active(light_source_params!($($params)*))
    };
}

macro_rules! skipped_light_source {
    ($($params:tt)*) => {
        LightSource2dCommon::skipped(light_source_params!($($params)*))
    };
}

mod beacon;
mod camera_optical;
mod format;
mod global;
mod light_group;
mod lightmap;
mod material;
mod particles;
mod roles;
#[cfg(test)]
mod tests;

pub use format::format_light_sources_2d;
pub(crate) use camera_optical::collect_camera_optical_candidates_from_light_sources_2d;

use roles::{color_rgba, light_source_roles, visual_source_availability_label};

pub fn collect_light_sources_2d(
    renderables: &[Renderable2dItem],
    beacons: &[amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand],
    global_lights: &[amigo_light_2d_plugin::GlobalLight2dCommand],
    lightmaps: &[amigo_light_2d_plugin::LightMap2dSourceCommand],
    light_groups: &[amigo_light_2d_plugin::LightGroup2dCommand],
    particles: &[amigo_particles_2d_plugin::Particle2dDrawCommand],
    camera_capture_input: Option<&CameraCaptureInput2d>,
) -> Vec<LightSource2dCommon> {
    let mut sources = Vec::new();
    material::collect_material_emissive_light_sources(renderables, &mut sources);
    beacon::collect_beacon_light_sources(beacons, &mut sources);
    global::collect_global_light_sources(global_lights, &mut sources);
    lightmap::collect_lightmap_sources(lightmaps, &mut sources);
    light_group::collect_light_group_sources(light_groups, global_lights, lightmaps, &mut sources);
    particles::collect_particle_light_sources(particles, &mut sources);
    camera_optical::collect_camera_capture_visual_sources(camera_capture_input, &mut sources);

    sources
}
