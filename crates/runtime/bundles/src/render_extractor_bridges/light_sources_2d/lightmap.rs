use super::*;

pub(super) fn collect_lightmap_sources(
    lightmaps: &[amigo_light_2d_plugin::LightMap2dSourceCommand],
    sources: &mut Vec<LightSource2dCommon>,
) {
    for lightmap in lightmaps.iter().take(MAX_LIGHTMAP_SOURCES) {
        if lightmap.channels.is_empty() {
            sources.push(skipped_light_source!(
                lightmap.entity_name.clone(),
                "LightMap2D",
                LightEmitterKind2d::LightMapSource,
                Some(lightmap.id.clone()),
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
            continue;
        }

        for channel in lightmap.channels.iter() {
            let layers = if channel.layers.is_empty() {
                "none".to_owned()
            } else {
                channel.layers.join(",")
            };
            sources.push(active_light_source!(
                lightmap.entity_name.clone(),
                "LightMap2D",
                LightEmitterKind2d::LightMapChannel,
                Some(format!("{}:{}", lightmap.id, channel.id)),
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
                    lightmap.id, channel.id
                ),
                None,
            ));
        }
    }
}
