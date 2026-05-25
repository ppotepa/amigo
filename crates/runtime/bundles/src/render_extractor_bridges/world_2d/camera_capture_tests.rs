use amigo_camera_core_plugin::api::CameraDepthMotion2d;
use amigo_camera_optics_plugin::api::{
    CameraOpticalCandidate2d, CameraOpticalCandidateStatus2d, CameraOpticalCoverage2d,
    CameraOpticalResponse2d,
};
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::RuntimeBuilder;

use super::camera_capture::build_camera_capture_input;

fn test_runtime() -> amigo_runtime::Runtime {
    RuntimeBuilder::default().build()
}

fn optical_candidate_with_roles(
    roles: &[&str],
    coverage: CameraOpticalCoverage2d,
    response: CameraOpticalResponse2d,
) -> CameraOpticalCandidate2d {
    CameraOpticalCandidate2d {
        owner: "test-source".to_owned(),
        component_kind: "LightGroup2D".to_owned(),
        render_layer: None,
        color_rgba: [1.0, 1.0, 1.0, 1.0],
        intensity: 1.0,
        response,
        coverage,
        roles: amigo_render_api::RenderContributionSet::from_pairs(
            roles.iter().map(|role| (*role, true)),
        ),
        status: CameraOpticalCandidateStatus2d::Active,
        reason: "camera_optical_candidate_active".to_owned(),
        position_px: None,
        target_ids: Vec::new(),
        trace: None,
    }
}

#[test]
fn camera_capture_input_includes_scene_color_depth_and_layers() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_2d_render_layer(amigo_2d_composition::RenderLayer2dCommand {
        source_mod: "rotten-club".to_owned(),
        id: "background.city".to_owned(),
        label: None,
        order: 0.0,
        visible: true,
        opacity: 1.0,
        depth: amigo_2d_composition::RenderDepth2d::default(),
        optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
    });

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        &[],
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );

    assert_eq!(
        input.color.kind,
        amigo_render_api::VisualSourceKind2d::SceneColor
    );
    assert_eq!(
        input.depth.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneDepth)
    );
    assert_eq!(
        input.layer_mask.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::LayerMask)
    );
    assert_eq!(input.layers.len(), 1);
    assert_eq!(input.layers[0].layer_id, "background.city");
    assert_eq!(
        input.layers[0].role,
        amigo_2d_spatial::OpticalLayerRole2d::WorldSurface
    );
    assert!(
        input
            .missing_source_kinds()
            .contains(&amigo_render_api::VisualSourceKind2d::SceneNormal)
    );
}

#[test]
fn camera_capture_input_applies_camera_z_to_distance_layers() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_2d_render_layer(amigo_2d_composition::RenderLayer2dCommand {
        source_mod: "rotten-club".to_owned(),
        id: "weather.rain.mid".to_owned(),
        label: None,
        order: 0.0,
        visible: true,
        opacity: 1.0,
        depth: amigo_2d_composition::RenderDepth2d {
            mode: amigo_2d_composition::RenderDepthMode2d::Distance,
            distance_m: Some(75.0),
            ..Default::default()
        },
        optical_role: amigo_2d_spatial::OpticalLayerRole2d::SceneMedium,
    });
    let depth_space = amigo_2d_spatial::DepthSpace2d::default();

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        &[],
        depth_space,
        amigo_camera_core_plugin::api::CameraDepthMotion2d {
            camera_z_m: 2.0,
            ..Default::default()
        },
    );

    assert_eq!(input.layers[0].distance_m, Some(75.0));
    assert_eq!(input.layers[0].effective_distance_m, Some(73.0));
    assert_eq!(
        input.layers[0].effective_z_depth,
        amigo_2d_spatial::distance_to_z_depth(73.0, depth_space)
    );
    assert_eq!(input.layers[0].z_depth, input.layers[0].effective_z_depth);
}

#[test]
fn camera_capture_input_does_not_set_highlight_for_lightmaps_without_candidates() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_render_contribution_2d(amigo_render_api::RenderContribution2d::lightmap_2d(
        amigo_render_api::RenderLightMap2dSource {
            source_mod: "rotten-club".to_owned(),
            owner_entity: "bar.lightmap".to_owned(),
            source_id: "bar.lightmap".to_owned(),
            source: amigo_render_api::RenderLightMap2dSourceRef {
                kind: amigo_render_api::RenderLightMap2dSourceKind::LayeredImage2d,
                entity_name: "bar.lightmap".to_owned(),
            },
            channels: Vec::new(),
        },
    ));

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        &[],
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );

    assert!(input.highlight.is_none());
    assert!(input.emissive.is_none());
}

#[test]
fn camera_capture_input_sets_highlight_from_active_optical_candidate() {
    let packet = WgpuRenderFramePacket::default();
    let candidates = vec![optical_candidate_with_roles(
        &[
            amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
            amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
        ],
        CameraOpticalCoverage2d::LightMapChannel {
            source: "neon-alley-lightmap".to_owned(),
            channel: "mid_neon".to_owned(),
        },
        CameraOpticalResponse2d {
            enabled: true,
            intensity: 0.8,
            bloom: 0.4,
            glare: 0.6,
            ..Default::default()
        },
    )];

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        candidates.as_slice(),
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );

    assert_eq!(
        input.highlight.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneHighlight)
    );
    assert_eq!(
        input.emissive.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneEmissive)
    );
}

#[test]
fn camera_capture_input_ignores_unsupported_optical_candidate() {
    let packet = WgpuRenderFramePacket::default();
    let candidates = vec![optical_candidate_with_roles(
        &[
            amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
            amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
        ],
        CameraOpticalCoverage2d::Unsupported {
            reason: "unsupported_for_test".to_owned(),
        },
        CameraOpticalResponse2d {
            enabled: true,
            intensity: 1.0,
            bloom: 1.0,
            glare: 1.0,
            ..Default::default()
        },
    )];

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        candidates.as_slice(),
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );

    assert!(input.highlight.is_none());
    assert!(input.emissive.is_none());
}

#[test]
fn camera_capture_input_uses_role_aware_candidate_targets() {
    let coverage = CameraOpticalCoverage2d::Hotspot {
        entity_name: "test".to_owned(),
        radius_px: 16.0,
    };

    let bloom_packet = WgpuRenderFramePacket::default();
    let bloom_candidates = vec![optical_candidate_with_roles(
        &[amigo_render_api::render_contribution_roles::BLOOM_SOURCE],
        coverage.clone(),
        CameraOpticalResponse2d {
            enabled: true,
            bloom: 1.0,
            ..Default::default()
        },
    )];
    let runtime = test_runtime();
    let bloom_input = build_camera_capture_input(
        &runtime,
        &bloom_packet,
        bloom_candidates.as_slice(),
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );
    assert!(bloom_input.highlight.is_none());
    assert_eq!(
        bloom_input.emissive.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneEmissive)
    );

    let camera_fx_packet = WgpuRenderFramePacket::default();
    let camera_fx_candidates = vec![optical_candidate_with_roles(
        &[amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE],
        coverage,
        CameraOpticalResponse2d {
            enabled: true,
            glare: 1.0,
            ..Default::default()
        },
    )];
    let camera_fx_input = build_camera_capture_input(
        &runtime,
        &camera_fx_packet,
        camera_fx_candidates.as_slice(),
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );
    assert_eq!(
        camera_fx_input.highlight.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneHighlight)
    );
    assert!(camera_fx_input.emissive.is_none());
}

#[test]
fn camera_capture_input_sets_wetness_from_active_wet_reflections_mask() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stacks(vec![amigo_composite_plugin::ScopedPostFx2dStack::new(
        "scene.weather",
        amigo_composite_plugin::PostFxScope2d::Frame,
        vec![amigo_composite_plugin::PostFx2dInstance::new(
            "wetness",
            amigo_render_api::post_fx_wet_reflections(
                amigo_composite_plugin::PostFxWetReflections2d {
                    enabled: true,
                    reflection_mask: "rotten-club/layered-images/neon-alley/reflection_mask.png"
                        .to_owned(),
                    ..Default::default()
                },
            ),
        )],
    )]);

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        &[],
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );

    assert_eq!(
        input.wetness.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneWetness)
    );
    assert_eq!(
        input.wetness.as_ref().map(|source| source.id.0.as_str()),
        Some("world.wetness")
    );
    assert_eq!(
        input.wetness.as_ref().map(|source| source.availability),
        Some(amigo_render_api::VisualSourceAvailability2d::Produced)
    );
}

#[test]
fn camera_capture_input_sets_normal_from_wet_reflections_noise_normal() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stacks(vec![amigo_composite_plugin::ScopedPostFx2dStack::new(
        "scene.weather",
        amigo_composite_plugin::PostFxScope2d::Frame,
        vec![amigo_composite_plugin::PostFx2dInstance::new(
            "wetness",
            amigo_render_api::post_fx_wet_reflections(
                amigo_composite_plugin::PostFxWetReflections2d {
                    enabled: true,
                    reflection_mask: "rotten-club/layered-images/neon-alley/reflection_mask.png"
                        .to_owned(),
                    noise_normal: Some(
                        "rotten-club/layered-images/neon-alley/rain-normal.png".to_owned(),
                    ),
                    ..Default::default()
                },
            ),
        )],
    )]);

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        &[],
        amigo_2d_spatial::DepthSpace2d::default(),
        amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
    );

    assert_eq!(
        input.normal.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneNormal)
    );
    assert_eq!(
        input.normal.as_ref().map(|source| source.id.0.as_str()),
        Some("world.normal")
    );
    assert_eq!(
        input.normal.as_ref().map(|source| source.availability),
        Some(amigo_render_api::VisualSourceAvailability2d::Produced)
    );
}

#[test]
fn camera_capture_input_sets_motion_from_active_shutter_blur() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stacks(vec![amigo_composite_plugin::ScopedPostFx2dStack::new(
        "camera.main",
        amigo_composite_plugin::PostFxScope2d::Frame,
        vec![amigo_composite_plugin::PostFx2dInstance::new(
            "motion",
            amigo_render_api::post_fx_shutter_blur(amigo_composite_plugin::ShutterBlur2d {
                exposure_seconds: 1.0 / 48.0,
                opacity: 0.8,
                shutter_angle: 180.0,
                ..Default::default()
            }),
        )],
    )]);

    let runtime = test_runtime();
    let input = build_camera_capture_input(
        &runtime,
        &packet,
        &[],
        amigo_2d_spatial::DepthSpace2d::default(),
        CameraDepthMotion2d::default(),
    );

    assert_eq!(
        input.motion.as_ref().map(|source| source.kind),
        Some(amigo_render_api::VisualSourceKind2d::SceneMotion)
    );
    assert_eq!(
        input.motion.as_ref().map(|source| source.id.0.as_str()),
        Some("world.motion")
    );
}
