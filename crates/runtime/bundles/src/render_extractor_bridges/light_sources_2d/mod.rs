use amigo_camera_optics_plugin::api::{
    CameraOpticalCandidate2d, CameraOpticalEmitterKind2d, CameraOpticalSource2d,
    CameraOpticalSourceStatus2d,
};
use amigo_material_api::Material2d;
use amigo_render_api::{
    CameraCaptureInput2d, LightContributionKind2d, LightEmitterKind2d, LightSource2dCommon,
    LightSource2dCommonParams, LightSourceStatus2d, RenderContribution2d, RenderContributionSet,
    VisualSourceAvailability2d,
};

use crate::render_extractor_bridges::visual_2d_items::Renderable2dItem;

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

mod camera_optical;
mod format;
mod material;
mod roles;
#[cfg(test)]
mod tests;

pub use format::format_light_sources_2d;
pub use camera_optical::collect_camera_optical_candidates_from_light_sources_2d;

use roles::{color_rgba, light_source_roles, visual_source_availability_label};

pub fn collect_light_sources_2d(
    renderables: &[Renderable2dItem],
    contributions_2d: &[RenderContribution2d],
    camera_capture_input: Option<&CameraCaptureInput2d>,
) -> Vec<LightSource2dCommon> {
    let mut sources = Vec::new();
    material::collect_material_emissive_light_sources(renderables, &mut sources);
    collect_contribution_light_sources(contributions_2d, &mut sources);
    camera_optical::collect_camera_capture_visual_sources(camera_capture_input, &mut sources);

    sources
}

fn collect_contribution_light_sources(
    contributions_2d: &[RenderContribution2d],
    sources: &mut Vec<LightSource2dCommon>,
) {
    for contribution in contributions_2d {
        if let Some(source) = contribution.as_light_source_2d() {
            sources.push(source.clone());
        }
        if let Some(lightmap) = contribution.as_lightmap_2d() {
            collect_lightmap_contribution_sources(lightmap, sources);
        }
        if let Some(group) = contribution.as_light_group_2d() {
            collect_light_group_contribution_sources(group, sources);
        }
    }
}

fn collect_lightmap_contribution_sources(
    lightmap: &amigo_render_api::RenderLightMap2dSource,
    sources: &mut Vec<LightSource2dCommon>,
) {
    if lightmap.channels.is_empty() {
        sources.push(skipped_light_source!(
            lightmap.owner_entity.clone(),
            "LightMap2D",
            LightEmitterKind2d::LightMapSource,
            Some(lightmap.source_id.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            vec![LightContributionKind2d::LightingEmit],
            "lightmap_source_without_channels",
            None,
        ));
        return;
    }

    for channel in &lightmap.channels {
        let layers = if channel.layers.is_empty() {
            "none".to_owned()
        } else {
            channel.layers.join(",")
        };
        sources.push(active_light_source!(
            lightmap.owner_entity.clone(),
            "LightMap2D",
            LightEmitterKind2d::LightMapChannel,
            Some(format!("{}:{}", lightmap.source_id, channel.id)),
            None,
            None,
            Some(1.0),
            Some(1.0),
            Some(1.0),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![LightContributionKind2d::LightingEmit],
            format!(
                "lightmap_channel source={} channel={} layers={layers}",
                lightmap.source_id, channel.id
            ),
            None,
        ));
    }
}

fn collect_light_group_contribution_sources(
    group: &amigo_render_api::RenderLightGroup2d,
    sources: &mut Vec<LightSource2dCommon>,
) {
    if group.sources.is_empty() {
        sources.push(skipped_light_source!(
            group.id.clone(),
            "LightGroup2D",
            LightEmitterKind2d::LightGroup,
            Some(group.id.clone()),
            None,
            Some(group.color_rgba),
            Some(group.intensity),
            Some(0.0),
            Some(1.0),
            Some(group.camera_response),
            None,
            None,
            None,
            None,
            None,
            light_group_contribution_roles(group),
            "light_group_without_sources",
            None,
        ));
        return;
    }

    for source in &group.sources {
        let emitter_id = match &source.kind {
            amigo_render_api::RenderLightGroupSourceKind2d::GlobalLight { id } => {
                Some(format!("{}:global:{}", group.id, id))
            }
            amigo_render_api::RenderLightGroupSourceKind2d::LightMapChannel {
                source,
                channel,
            } => Some(format!("{}:lightmap:{}:{}", group.id, source, channel)),
        };
        let effective_intensity = group.intensity * source.response.max(0.0);
        sources.push(active_light_source!(
            group.id.clone(),
            "LightGroup2D",
            LightEmitterKind2d::LightGroup,
            emitter_id,
            None,
            Some(group.color_rgba),
            Some(group.intensity),
            Some(effective_intensity),
            Some(source.response.max(0.0)),
            Some(group.camera_response),
            None,
            None,
            None,
            None,
            None,
            light_group_contribution_roles(group),
            "light_group_contribution",
            None,
        ));
    }
}

fn light_group_contribution_roles(
    group: &amigo_render_api::RenderLightGroup2d,
) -> Vec<LightContributionKind2d> {
    let mut contributions = Vec::new();
    if group
        .contributions
        .enabled_or(amigo_render_api::render_contribution_roles::LIGHTING_EMIT, true)
    {
        contributions.push(LightContributionKind2d::LightingEmit);
    }
    if group
        .contributions
        .enabled_or(amigo_render_api::render_contribution_roles::BLOOM_SOURCE, false)
    {
        contributions.push(LightContributionKind2d::BloomSource);
    }
    if group
        .contributions
        .enabled_or(amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE, false)
    {
        contributions.push(LightContributionKind2d::CameraFxSource);
    }
    contributions
}
