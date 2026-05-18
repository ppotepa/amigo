use amigo_render_api::{
    RenderContributionDecision, RenderContributionStatus, render_contribution_roles as roles,
};

// Renderable2dCommon lives in render-api because it is backend-neutral.
// Renderable2dPayload stays in render-wgpu while WgpuRenderFramePacket owns
// backend draw payloads. Extractors use this module as their stable adapter API.
pub use amigo_render_api::{RenderSpace2d, Renderable2dCommon, Renderable2dKind};
pub use amigo_render_wgpu::{Renderable2dItem, Renderable2dPayload};

pub fn supported_renderable_2d_component_kinds() -> &'static [&'static str] {
    amigo_render_wgpu::supported_renderable_2d_component_kinds()
}

pub fn render_contribution_decisions_summary(
    beacons: &[amigo_2d_lighting_beacon::BeaconLight2dDrawCommand],
) -> Option<String> {
    if beacons.is_empty() {
        return None;
    }

    let decisions = collect_beacon_render_contribution_decisions(beacons);
    Some(format_render_contribution_decisions(&decisions))
}

pub fn collect_beacon_render_contribution_decisions(
    beacons: &[amigo_2d_lighting_beacon::BeaconLight2dDrawCommand],
) -> Vec<RenderContributionDecision> {
    let mut decisions = Vec::new();
    for beacon in beacons.iter().take(8) {
        push_beacon_decision(&mut decisions, beacon, roles::OVERLAY_VISIBLE, true);
        push_beacon_decision(&mut decisions, beacon, roles::RELIGHT_PLATE, true);
        push_beacon_decision(&mut decisions, beacon, roles::BLOOM_SOURCE, true);
        push_beacon_decision(&mut decisions, beacon, roles::CAMERA_FX_SOURCE, true);
    }
    decisions
}

pub fn format_render_contribution_decisions(
    decisions: &[RenderContributionDecision],
) -> String {
    let mut lines = Vec::new();
    lines.push("render.contributions:".to_owned());

    if decisions.is_empty() {
        lines.push("none".to_owned());
        return lines.join("\n");
    }

    for decision in decisions {
        let status = match decision.status {
            RenderContributionStatus::Active => "active",
            RenderContributionStatus::Skipped => "skipped",
        };

        lines.push(format!(
            "owner={} component={} role {}: {} reason={}",
            decision.owner,
            decision.component,
            decision.role.as_str(),
            status,
            decision.reason
        ));
    }

    lines.join("\n")
}

fn push_beacon_decision(
    decisions: &mut Vec<RenderContributionDecision>,
    beacon: &amigo_2d_lighting_beacon::BeaconLight2dDrawCommand,
    role: &'static str,
    default_enabled: bool,
) {
    let active = beacon.render_contributions.enabled_or(role, default_enabled);
    let reason = if active {
        "enabled_by_authoring_or_default"
    } else {
        "disabled_by_authoring"
    };

    let decision = if active {
        RenderContributionDecision::active(
            beacon.entity_name.clone(),
            "BeaconLight2D",
            role,
            reason,
        )
    } else {
        RenderContributionDecision::skipped(
            beacon.entity_name.clone(),
            "BeaconLight2D",
            role,
            reason,
        )
    };

    decisions.push(decision);
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_builtin_renderable_2d_component_has_visual_item_adapter() {
        let renderable_kinds = amigo_scene::builtin_renderable_2d_component_kinds();
        let supported = super::supported_renderable_2d_component_kinds();

        for kind in renderable_kinds {
            assert!(
                supported.contains(kind),
                "Renderable2D component {kind} must be collected as Renderable2dItem"
            );
        }
    }
}
