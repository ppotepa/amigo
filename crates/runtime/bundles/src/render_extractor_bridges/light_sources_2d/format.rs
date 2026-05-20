use amigo_render_api::{LightContributionKind2d, LightSource2dCommon, LightSourceStatus2d};

pub fn format_light_sources_2d(sources: &[LightSource2dCommon]) -> String {
    let mut lines = vec!["render.light.sources:".to_owned()];

    if sources.is_empty() {
        lines.push("none".to_owned());
        return lines.join("\n");
    }

    for source in sources {
        let contributions = if source.contributions.is_empty() {
            "none".to_owned()
        } else {
            source
                .contributions
                .iter()
                .map(|kind| kind.as_str())
                .collect::<Vec<_>>()
                .join(",")
        };
        let used_by = if source.status == LightSourceStatus2d::Active
            && source
                .contributions
                .contains(&LightContributionKind2d::RelightPlate)
        {
            "plate_relight"
        } else {
            "none"
        };

        lines.push(format!(
            "owner={} component={} emitter={} status={} reason={}",
            source.owner,
            source.component_kind,
            source.emitter_kind.as_str(),
            source.status.as_str(),
            source.reason
        ));
        lines.push(format!(
            "id={} layer={} color={} intensity={} effective_intensity={} response={} bloom={} radius_px={} falloff={} distance_m={} z_depth={} contributions={} used_by={}",
            source.emitter_id.as_deref().unwrap_or("-"),
            source.render_layer.as_deref().unwrap_or("-"),
            format_color(source.color_rgba),
            format_opt(source.intensity),
            format_opt(source.effective_intensity),
            format_opt(source.response),
            format_opt(source.bloom),
            format_opt(source.radius_px),
            format_opt(source.falloff),
            format_opt(source.distance_m),
            format_opt(source.z_depth),
            contributions,
            used_by
        ));
    }

    lines.join("\n")
}

fn format_color(value: Option<[f32; 4]>) -> String {
    match value {
        Some([r, g, b, a]) => format!("{r:.3},{g:.3},{b:.3},{a:.3}"),
        None => "-".to_owned(),
    }
}

fn format_opt(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "-".to_owned())
}
