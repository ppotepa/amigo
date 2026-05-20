use super::*;

pub(super) fn collect_beacon_light_sources(
    beacons: &[amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand],
    sources: &mut Vec<LightSource2dCommon>,
) {
    for beacon in beacons.iter().take(MAX_BEACON_LIGHT_SOURCES) {
        let position_px = Some([beacon.center.x, beacon.center.y]);
        let mut contributions = Vec::new();
        if beacon.render_contributions.enabled_or(
            amigo_render_api::render_contribution_roles::RELIGHT_PLATE,
            true,
        ) {
            contributions.push(LightContributionKind2d::RelightPlate);
        }
        if beacon.render_contributions.enabled_or(
            amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
            true,
        ) {
            contributions.push(LightContributionKind2d::BloomSource);
        }
        if beacon.render_contributions.enabled_or(
            amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
            true,
        ) {
            contributions.push(LightContributionKind2d::CameraFxSource);
        }
        let status = if contributions.is_empty() {
            LightSourceStatus2d::Skipped
        } else {
            LightSourceStatus2d::Active
        };
        let reason = if contributions.is_empty() {
            "all_light_roles_disabled".to_owned()
        } else {
            "active_light_emitter".to_owned()
        };

        let source = match status {
            LightSourceStatus2d::Active => active_light_source!(
                beacon.entity_name.clone(),
                "BeaconLight2D",
                LightEmitterKind2d::Beacon,
                None,
                Some(beacon.render_layer.clone()),
                Some(color_rgba(beacon.color)),
                Some(beacon.intensity),
                Some(beacon.intensity * beacon.color.a),
                Some(1.0),
                Some(beacon.camera_response),
                Some(beacon.bloom),
                Some(beacon.halo_radius_px.max(beacon.core_radius_px)),
                None,
                beacon.distance_m,
                beacon.z_depth,
                contributions,
                reason,
                position_px,
            ),
            LightSourceStatus2d::Skipped => skipped_light_source!(
                beacon.entity_name.clone(),
                "BeaconLight2D",
                LightEmitterKind2d::Beacon,
                None,
                Some(beacon.render_layer.clone()),
                Some(color_rgba(beacon.color)),
                Some(beacon.intensity),
                Some(beacon.intensity * beacon.color.a),
                Some(1.0),
                Some(beacon.camera_response),
                Some(beacon.bloom),
                Some(beacon.halo_radius_px.max(beacon.core_radius_px)),
                None,
                beacon.distance_m,
                beacon.z_depth,
                contributions,
                reason,
                position_px,
            ),
        };
        sources.push(source);
    }
}
