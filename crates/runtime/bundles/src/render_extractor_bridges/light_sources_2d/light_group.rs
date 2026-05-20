use super::*;

pub(super) fn collect_light_group_sources(
    light_groups: &[amigo_light_2d_plugin::LightGroup2dCommand],
    global_lights: &[amigo_light_2d_plugin::GlobalLight2dCommand],
    lightmaps: &[amigo_light_2d_plugin::LightMap2dSourceCommand],
    mut sources: &mut Vec<LightSource2dCommon>,
) {
    for group in light_groups.iter().take(MAX_LIGHT_GROUP_SOURCES) {
        if group.sources.is_empty() {
            sources.push(skipped_light_source!(
                group.id.clone(),
                "LightGroup2D",
                LightEmitterKind2d::LightGroup,
                Some(group.id.clone()),
                None,
                Some(color_rgba(group.color)),
                Some(group.intensity),
                Some(0.0),
                Some(1.0),
                Some(group.camera_response),
                None,
                None,
                None,
                None,
                None,
                light_group_contributions(group),
                "light_group_without_sources",
                None,
            ));
            continue;
        }

        for source in group.sources.iter() {
            match &source.kind {
                amigo_light_2d_plugin::LightGroup2dSourceKind::GlobalLight { id } => {
                    let global = global_lights.iter().find(|light| &light.id == id);
                    let (status, effective_intensity, reason) = match global {
                        Some(light) => (
                            LightSourceStatus2d::Active,
                            group.intensity * source.response * light.intensity,
                            format!("light_group_global_light group={} source={id}", group.id),
                        ),
                        None => (
                            LightSourceStatus2d::Skipped,
                            0.0,
                            format!("missing_global_light_source group={} source={id}", group.id),
                        ),
                    };
                    push_light_group_source(
                        &mut sources,
                        group,
                        Some(format!("{}:global:{}", group.id, id)),
                        source.response,
                        effective_intensity,
                        status,
                        reason,
                    );
                }
                amigo_light_2d_plugin::LightGroup2dSourceKind::LightMapChannel {
                    source: source_id,
                    channel,
                } => {
                    let found = lightmaps.iter().any(|lightmap| {
                        &lightmap.id == source_id
                            && lightmap.channels.iter().any(|entry| &entry.id == channel)
                    });
                    let (status, effective_intensity, reason) = if found {
                        (
                            LightSourceStatus2d::Active,
                            group.intensity * source.response,
                            format!(
                                "light_group_lightmap_channel group={} source={} channel={}",
                                group.id, source_id, channel
                            ),
                        )
                    } else {
                        (
                            LightSourceStatus2d::Skipped,
                            0.0,
                            format!(
                                "missing_lightmap_channel_source group={} source={} channel={}",
                                group.id, source_id, channel
                            ),
                        )
                    };
                    push_light_group_source(
                        &mut sources,
                        group,
                        Some(format!("{}:lightmap:{}:{}", group.id, source_id, channel)),
                        source.response,
                        effective_intensity,
                        status,
                        reason,
                    );
                }
            }
        }
    }
}

fn push_light_group_source(
    sources: &mut Vec<LightSource2dCommon>,
    group: &amigo_light_2d_plugin::LightGroup2dCommand,
    emitter_id: Option<String>,
    response: f32,
    effective_intensity: f32,
    status: LightSourceStatus2d,
    reason: String,
) {
    let common = match status {
        LightSourceStatus2d::Active => active_light_source!(
            group.id.clone(),
            "LightGroup2D",
            LightEmitterKind2d::LightGroup,
            emitter_id,
            None,
            Some(color_rgba(group.color)),
            Some(group.intensity),
            Some(effective_intensity),
            Some(response),
            Some(group.camera_response),
            None,
            None,
            None,
            None,
            None,
            light_group_contributions(group),
            reason,
            None,
        ),
        LightSourceStatus2d::Skipped => skipped_light_source!(
            group.id.clone(),
            "LightGroup2D",
            LightEmitterKind2d::LightGroup,
            emitter_id,
            None,
            Some(color_rgba(group.color)),
            Some(group.intensity),
            Some(effective_intensity),
            Some(response),
            Some(group.camera_response),
            None,
            None,
            None,
            None,
            None,
            light_group_contributions(group),
            reason,
            None,
        ),
    };
    sources.push(common);
}

fn light_group_contributions(
    group: &amigo_light_2d_plugin::LightGroup2dCommand,
) -> Vec<LightContributionKind2d> {
    let mut contributions = Vec::new();
    if group.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::LIGHTING_EMIT,
        true,
    ) {
        contributions.push(LightContributionKind2d::LightingEmit);
    }
    if group.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
        false,
    ) {
        contributions.push(LightContributionKind2d::BloomSource);
    }
    if group.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
        false,
    ) {
        contributions.push(LightContributionKind2d::CameraFxSource);
    }
    contributions
}
