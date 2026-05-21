use super::{
    GlobalLight2dCommand, GlobalLight2dSceneService, LightGroup2dCommand, LightGroup2dSceneService,
    LightMap2dSceneService, LightMap2dSourceCommand,
};
use amigo_render_api::{
    LightContributionKind2d, LightEmitterKind2d, LightSource2dCommon, LightSource2dCommonParams,
    RenderContribution2d, RenderExtractionOutput2d, RenderLightGroup2d, RenderLightGroupSource2d,
    RenderLightGroupSourceKind2d, RenderLightMap2dChannel, RenderLightMap2dSource,
    RenderLightMap2dSourceKind, RenderLightMap2dSourceRef,
};

pub const LIGHTING_2D_EXTRACTOR_ID: &str = "lighting_2d";

pub struct Lighting2dRenderExtractionContext<'a> {
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub lightmap2d_scene_service: &'a LightMap2dSceneService,
    pub light_group2d_scene_service: &'a LightGroup2dSceneService,
}

#[derive(Debug, Default, Clone)]
pub struct Lighting2dRenderCommands {
    pub global_lights: Vec<GlobalLight2dCommand>,
    pub lightmaps: Vec<LightMap2dSourceCommand>,
    pub light_groups: Vec<LightGroup2dCommand>,
}

pub struct Lighting2dRenderExtractor;

impl Lighting2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        LIGHTING_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: Lighting2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        let commands = extract_lighting2d_render_commands(ctx);
        for command in &commands.global_lights {
            output.push_render_contribution_2d(global_light_command_to_render_contribution(
                command,
            ));
        }
        for contribution in lightmap_commands_to_render_contributions(&commands.lightmaps) {
            output.push_render_contribution_2d(contribution);
        }
        for contribution in light_group_commands_to_render_contributions(
            &commands.light_groups,
            &commands.global_lights,
            &commands.lightmaps,
        ) {
            output.push_render_contribution_2d(contribution);
        }
    }
}

pub fn extract_lighting2d_render_commands(
    ctx: Lighting2dRenderExtractionContext<'_>,
) -> Lighting2dRenderCommands {
    Lighting2dRenderCommands {
        global_lights: ctx.global_light2d_scene_service.commands(),
        lightmaps: ctx.lightmap2d_scene_service.commands(),
        light_groups: ctx.light_group2d_scene_service.commands(),
    }
}

pub fn global_light_command_to_render_contribution(
    command: &GlobalLight2dCommand,
) -> RenderContribution2d {
    RenderContribution2d::light_source_2d(global_light_command_to_light_source(command))
}

pub fn global_light_command_to_light_source(command: &GlobalLight2dCommand) -> LightSource2dCommon {
    LightSource2dCommon::active(LightSource2dCommonParams {
        owner: command.entity_name.clone(),
        component_kind: "GlobalLight2D".to_owned(),
        emitter_kind: LightEmitterKind2d::GlobalLight,
        emitter_id: Some(command.id.clone()),
        render_layer: None,
        color_rgba: Some([command.color.r, command.color.g, command.color.b, command.color.a]),
        intensity: Some(command.intensity),
        effective_intensity: Some(command.intensity * command.color.a),
        response: Some(1.0),
        camera_response: None,
        bloom: None,
        radius_px: None,
        falloff: None,
        distance_m: None,
        z_depth: None,
        contributions: vec![LightContributionKind2d::LightingEmit],
        reason: "global_light_command".to_owned(),
        position_px: None,
    })
}

pub fn lightmap_commands_to_render_contributions(
    lightmaps: &[LightMap2dSourceCommand],
) -> Vec<RenderContribution2d> {
    lightmaps
        .iter()
        .map(|lightmap| {
            RenderContribution2d::lightmap_2d(RenderLightMap2dSource {
                source_mod: lightmap.source_mod.clone(),
                owner_entity: lightmap.entity_name.clone(),
                source_id: lightmap.id.clone(),
                source: RenderLightMap2dSourceRef {
                    kind: match lightmap.source.kind {
                        super::LightMap2dSourceKind::LayeredImage2d => {
                            RenderLightMap2dSourceKind::LayeredImage2d
                        }
                    },
                    entity_name: lightmap.source.entity_name.clone(),
                },
                channels: lightmap
                    .channels
                    .iter()
                    .map(|channel| RenderLightMap2dChannel {
                        id: channel.id.clone(),
                        layers: channel.layers.clone(),
                    })
                    .collect(),
            })
        })
        .collect()
}

pub fn light_group_commands_to_render_contributions(
    light_groups: &[LightGroup2dCommand],
    global_lights: &[GlobalLight2dCommand],
    lightmaps: &[LightMap2dSourceCommand],
) -> Vec<RenderContribution2d> {
    let known_globals = global_lights
        .iter()
        .map(|light| light.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let known_lightmap_channels = lightmaps
        .iter()
        .flat_map(|lightmap| {
            lightmap
                .channels
                .iter()
                .map(move |channel| (lightmap.id.as_str(), channel.id.as_str()))
        })
        .collect::<std::collections::BTreeSet<_>>();

    light_groups
        .iter()
        .map(|group| {
            RenderContribution2d::light_group_2d(RenderLightGroup2d {
                source_mod: group.source_mod.clone(),
                id: group.id.clone(),
                label: group.label.clone(),
                color_rgba: [group.color.r, group.color.g, group.color.b, group.color.a],
                intensity: group.intensity,
                contributions: group.render_contributions.clone(),
                camera_response: group.camera_response,
                sources: group
                    .sources
                    .iter()
                    .filter_map(|source| match &source.kind {
                        super::LightGroup2dSourceKind::GlobalLight { id } => known_globals
                            .contains(id.as_str())
                            .then_some(RenderLightGroupSource2d {
                                kind: RenderLightGroupSourceKind2d::GlobalLight { id: id.clone() },
                                response: source.response,
                            }),
                        super::LightGroup2dSourceKind::LightMapChannel {
                            source: source_id,
                            channel,
                        } => known_lightmap_channels
                            .contains(&(source_id.as_str(), channel.as_str()))
                            .then_some(RenderLightGroupSource2d {
                                kind: RenderLightGroupSourceKind2d::LightMapChannel {
                                    source: source_id.clone(),
                                    channel: channel.clone(),
                                },
                                response: source.response,
                            }),
                    })
                    .collect(),
            })
        })
        .collect()
}

fn light_group_contributions(group: &LightGroup2dCommand) -> Vec<LightContributionKind2d> {
    let mut contributions = Vec::new();
    if group
        .render_contributions
        .enabled_or(amigo_render_api::render_contribution_roles::LIGHTING_EMIT, true)
    {
        contributions.push(LightContributionKind2d::LightingEmit);
    }
    if group
        .render_contributions
        .enabled_or(amigo_render_api::render_contribution_roles::BLOOM_SOURCE, false)
    {
        contributions.push(LightContributionKind2d::BloomSource);
    }
    if group
        .render_contributions
        .enabled_or(amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE, false)
    {
        contributions.push(LightContributionKind2d::CameraFxSource);
    }
    contributions
}
