use super::*;

pub(super) fn collect_global_light_sources(
    global_lights: &[amigo_light_2d_plugin::GlobalLight2dCommand],
    sources: &mut Vec<LightSource2dCommon>,
) {
    for global_light in global_lights.iter().take(MAX_GLOBAL_LIGHT_SOURCES) {
        sources.push(active_light_source!(
            global_light.entity_name.clone(),
            "GlobalLight2D",
            LightEmitterKind2d::GlobalLight,
            Some(global_light.id.clone()),
            None,
            Some(color_rgba(global_light.color)),
            Some(global_light.intensity),
            Some(global_light.intensity * global_light.color.a),
            Some(1.0),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![LightContributionKind2d::LightingEmit],
            "global_light_command",
            None,
        ));
    }
}
