use amigo_render_api::{
    LightContributionKind2d, LightSource2dCommon, LightSourceStatus2d, RenderContributionDecision,
    RenderContributionSet, RenderContributionStatus, render_contribution_roles as roles,
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
    renderables: &[Renderable2dItem],
    light_sources: &[LightSource2dCommon],
) -> Option<String> {
    let decisions = collect_render_contribution_decisions_2d(renderables, light_sources);
    if decisions.is_empty() {
        return None;
    }

    Some(format_render_contribution_decisions(&decisions))
}

pub fn collect_render_contribution_decisions_2d(
    renderables: &[Renderable2dItem],
    light_sources: &[LightSource2dCommon],
) -> Vec<RenderContributionDecision> {
    let mut decisions = Vec::new();
    for item in renderables.iter().take(64) {
        push_renderable_role(
            &mut decisions,
            &item.common.owner_entity,
            &item.common.component_kind,
            renderable_contributions(item),
            roles::WORLD_COLOR,
            true,
        );
        match &item.payload {
            Renderable2dPayload::Text(_)
            | Renderable2dPayload::Sprite(_)
            | Renderable2dPayload::Vector(_) => {
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::MATERIAL_MASK,
                    false,
                );
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::OPTICS_REFRACT,
                    false,
                );
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::BLOOM_SOURCE,
                    false,
                );
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::CAMERA_FX_SOURCE,
                    false,
                );
            }
            Renderable2dPayload::Beacon(_) => {
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::OVERLAY_VISIBLE,
                    true,
                );
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::RELIGHT_PLATE,
                    true,
                );
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::BLOOM_SOURCE,
                    true,
                );
                push_renderable_role(
                    &mut decisions,
                    &item.common.owner_entity,
                    &item.common.component_kind,
                    renderable_contributions(item),
                    roles::CAMERA_FX_SOURCE,
                    true,
                );
            }
            _ => {}
        }
    }

    for source in light_sources.iter().take(64) {
        if source.contributions.is_empty() && source.status == LightSourceStatus2d::Skipped {
            decisions.push(RenderContributionDecision::skipped(
                source.owner.clone(),
                source.component_kind.clone(),
                roles::LIGHTING_EMIT,
                source.reason.clone(),
            ));
            continue;
        }
        for contribution in &source.contributions {
            let role = light_contribution_role(*contribution);
            let decision = if source.status == LightSourceStatus2d::Active {
                RenderContributionDecision::active(
                    source.owner.clone(),
                    source.component_kind.clone(),
                    role,
                    "enabled_by_light_source",
                )
            } else {
                RenderContributionDecision::skipped(
                    source.owner.clone(),
                    source.component_kind.clone(),
                    role,
                    source.reason.clone(),
                )
            };
            decisions.push(decision);
        }
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

fn renderable_contributions(item: &Renderable2dItem) -> Option<&RenderContributionSet> {
    match &item.payload {
        Renderable2dPayload::Text(command) => Some(&command.render_contributions),
        Renderable2dPayload::Sprite(command) => Some(&command.render_contributions),
        Renderable2dPayload::Vector(command) => Some(&command.render_contributions),
        Renderable2dPayload::Beacon(command) => Some(&command.render_contributions),
        _ => None,
    }
}

fn push_renderable_role(
    decisions: &mut Vec<RenderContributionDecision>,
    owner: &str,
    component: &str,
    set: Option<&RenderContributionSet>,
    role: &'static str,
    default_enabled: bool,
) {
    let active = set
        .map(|set| set.enabled_or(role, default_enabled))
        .unwrap_or(default_enabled);
    let reason = if active {
        "enabled_by_authoring_or_default"
    } else {
        "disabled_by_authoring"
    };

    let decision = if active {
        RenderContributionDecision::active(
            owner,
            component,
            role,
            reason,
        )
    } else {
        RenderContributionDecision::skipped(
            owner,
            component,
            role,
            reason,
        )
    };

    decisions.push(decision);
}

fn light_contribution_role(contribution: LightContributionKind2d) -> &'static str {
    match contribution {
        LightContributionKind2d::LightingEmit => roles::LIGHTING_EMIT,
        LightContributionKind2d::RelightPlate => roles::RELIGHT_PLATE,
        LightContributionKind2d::BloomSource => roles::BLOOM_SOURCE,
        LightContributionKind2d::CameraFxSource => roles::CAMERA_FX_SOURCE,
        LightContributionKind2d::EmissiveBuffer => "emissive_buffer",
    }
}

#[cfg(test)]
mod tests {
    use amigo_assets::AssetKey;
    use amigo_core::TypedId;
    use amigo_math::{Transform2, Vec2};
    use amigo_render_api::{
        LightContributionKind2d, LightEmitterKind2d, LightSource2dCommon, RenderContributionSet,
        RenderSpace2d, Renderable2dCommon, Renderable2dKind,
    };
    use amigo_render_wgpu::{Renderable2dItem, Renderable2dPayload};

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

    #[test]
    fn render_contributions_summary_includes_renderables_and_light_sources() {
        let renderable = Renderable2dItem {
            common: Renderable2dCommon {
                owner_entity: "title".to_owned(),
                component_kind: "Text2D".to_owned(),
                render_space: RenderSpace2d::World,
                render_layer: "title.depth2d".to_owned(),
                z_index: 0.0,
                kind: Renderable2dKind::Text,
            },
            payload: Renderable2dPayload::Text(amigo_2d_text::Text2dDrawCommand {
                entity_id: TypedId::new(1),
                entity_name: "title".to_owned(),
                render_layer: "title.depth2d".to_owned(),
                text: amigo_2d_text::Text2d {
                    content: "ROTTEN CLUB".to_owned(),
                    font: AssetKey::new("test/font"),
                    bounds: Vec2::new(128.0, 32.0),
                    transform: Transform2::default(),
                    style: amigo_2d_text::Text2dStyle::default(),
                    post_fx_host_id: None,
                },
                z_index: 0.0,
                material: None,
                render_contributions: RenderContributionSet::from_pairs([
                    (amigo_render_api::render_contribution_roles::MATERIAL_MASK, true),
                ]),
            }),
        };
        let light_source = LightSource2dCommon::active(
            "neon.mid",
            "LightGroup2D",
            LightEmitterKind2d::LightGroup,
            Some("neon.mid:lightmap:neon-alley-lightmap:mid_neon".to_owned()),
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
            None,
            vec![LightContributionKind2d::LightingEmit],
            "light_group_lightmap_channel",
            None,
        );

        let summary = super::render_contribution_decisions_summary(&[renderable], &[light_source])
            .expect("summary should include renderable and light source decisions");

        assert!(summary.contains("Text2D"));
        assert!(summary.contains("material.mask"));
        assert!(summary.contains("LightGroup2D"));
        assert!(summary.contains("lighting.emit"));
    }
}
