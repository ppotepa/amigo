use super::*;

pub(super) fn collect_material_emissive_light_sources(
    renderables: &[Renderable2dItem],
    sources: &mut Vec<LightSource2dCommon>,
) {
    for item in renderables.iter().take(MAX_MATERIAL_EMISSIVE_LIGHT_SOURCES) {
        let Some((material, color_rgba)) = material_light_payload(item) else {
            continue;
        };
        let response = material.camera_response.normalized();
        let has_camera_response = response.enabled
            && (response.intensity > 0.0
                || response.bloom > 0.0
                || response.glare > 0.0
                || response.dirt_response > 0.0
                || response.halation > 0.0);
        if !has_camera_response {
            sources.push(skipped_light_source!(
                item.common.owner_entity.clone(),
                item.common.component_kind.clone(),
                LightEmitterKind2d::EmissiveMaterial,
                Some(format!("material:{}", item.common.owner_entity)),
                Some(item.common.render_layer.clone()),
                Some(color_rgba),
                Some(0.0),
                Some(0.0),
                None,
                Some(response),
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                "material_emissive_no_camera_response",
                None,
            ));
            continue;
        }

        let mut contributions = Vec::new();
        if response.bloom > 0.0 || response.intensity > 0.0 {
            contributions.push(LightContributionKind2d::BloomSource);
            contributions.push(LightContributionKind2d::EmissiveBuffer);
        }
        if response.intensity > 0.0
            || response.glare > 0.0
            || response.ghosting > 0.0
            || response.streaks > 0.0
            || response.dirt_response > 0.0
            || response.halation > 0.0
        {
            contributions.push(LightContributionKind2d::CameraFxSource);
        }
        let intensity = response
            .intensity
            .max(response.glare)
            .max(response.bloom)
            .max(response.halation);
        sources.push(active_light_source!(
            item.common.owner_entity.clone(),
            item.common.component_kind.clone(),
            LightEmitterKind2d::EmissiveMaterial,
            Some(format!("material:{}", item.common.owner_entity)),
            Some(item.common.render_layer.clone()),
            Some(color_rgba),
            Some(intensity),
            Some(intensity),
            None,
            Some(response),
            Some(if response.bloom > 0.0 || response.intensity > 0.0 {
                intensity
            } else {
                0.0
            }),
            None,
            None,
            None,
            None,
            contributions,
            "material_emissive_camera_response",
            None,
        ));
    }
}

fn material_light_payload(item: &Renderable2dItem) -> Option<(Material2d, [f32; 4])> {
    match &item.payload {
        Renderable2dPayload::Text(command) => Some((
            command.material?,
            [
                command.text.style.color.r,
                command.text.style.color.g,
                command.text.style.color.b,
                command.text.style.color.a * command.text.style.opacity,
            ],
        )),
        Renderable2dPayload::Sprite(command) => Some((command.material?, [1.0, 1.0, 1.0, 1.0])),
        Renderable2dPayload::Vector(command) => Some((
            command.material?,
            color_rgba(
                command
                    .shape
                    .style
                    .fill_color
                    .unwrap_or(command.shape.style.stroke_color),
            ),
        )),
        _ => None,
    }
}
