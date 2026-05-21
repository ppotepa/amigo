use super::world::WorldRenderContext;
use super::world_filters::{
    LayeredImagePartFilter, WorldLayerFilter, WorldObjectFilter, WorldPassLoad,
};
use super::world_selection::WorldRenderSelection;
use super::*;

pub(super) fn try_execute_camera_debug_view(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    source: &wgpu::TextureView,
    visual_sources: Option<&crate::renderer::service::WgpuCameraVisualSources2d>,
    host_id: &amigo_composite_plugin::PostFxHost2dId,
    effect_id: &amigo_composite_plugin::PostFx2dId,
    feature_id: &str,
) -> AmigoResult<bool> {
    if !request.camera_debug_view.wants_visual_source_debug() {
        return Ok(false);
    }

    match request.camera_debug_view.as_str() {
        "camera.layer_optical_roles" => {
            if !copy_layer_roles_target_or_fallback(renderer, target)? {
                render_layer_roles_debug_source(renderer, request, target)?;
            }
            Ok(true)
        }
        "camera.layer_mask" => {
            if !copy_visual_source_target_or_fallback(
                renderer,
                target,
                amigo_render_api::VisualSourceKind2d::LayerMask,
            )? {
                render_layer_mask_visual_source(renderer, request, target)?;
            }
            Ok(true)
        }
        "camera.scene_normal" => {
            render_visual_source_debug_view(
                renderer,
                request,
                target,
                source,
                visual_sources,
                amigo_render_api::VisualSourceKind2d::SceneNormal,
                host_id,
                effect_id,
                feature_id,
            )?;
            Ok(true)
        }
        "camera.scene_wetness" => {
            render_visual_source_debug_view(
                renderer,
                request,
                target,
                source,
                visual_sources,
                amigo_render_api::VisualSourceKind2d::SceneWetness,
                host_id,
                effect_id,
                feature_id,
            )?;
            Ok(true)
        }
        "camera.scene_emissive" => {
            render_visual_source_debug_view(
                renderer,
                request,
                target,
                source,
                visual_sources,
                amigo_render_api::VisualSourceKind2d::SceneEmissive,
                host_id,
                effect_id,
                feature_id,
            )?;
            Ok(true)
        }
        "camera.scene_highlight" => {
            render_visual_source_debug_view(
                renderer,
                request,
                target,
                source,
                visual_sources,
                amigo_render_api::VisualSourceKind2d::SceneHighlight,
                host_id,
                effect_id,
                feature_id,
            )?;
            Ok(true)
        }
        "camera.scene_motion" => {
            render_visual_source_debug_view(
                renderer,
                request,
                target,
                source,
                visual_sources,
                amigo_render_api::VisualSourceKind2d::SceneMotion,
                host_id,
                effect_id,
                feature_id,
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn camera_capture_layers<'a>(
    request: &'a WgpuFrameRenderRequest<'a>,
) -> Option<&'a [amigo_render_api::ResolvedLayerOptics2d]> {
    request
        .camera_capture_input_2d
        .as_ref()
        .map(|input| input.layers.as_slice())
}

pub(super) fn render_layer_roles_debug_source(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    if request
        .visual_source_flags_2d
        .is_some_and(|flags| !flags.layer_roles_generated)
    {
        return renderer.clear_offscreen_to_color(
            target,
            color_to_wgpu(crate::renderer::service::fallback_color_for(
                amigo_render_api::VisualSourceKind2d::Debug,
            )),
        );
    }
    renderer.clear_offscreen_to_color(target, wgpu::Color::BLACK)?;
    let Some(layers) = camera_capture_layers(request) else {
        return Ok(());
    };
    for layer in layers {
        let single_layer = BTreeSet::from([layer.layer_id.clone()]);
        let mut layer_source = super::offscreen_ops::compatible_offscreen_target(
            target,
            "amigo-debug-layer-role-source",
        );
        world::execute_world_to_offscreen(
            renderer,
            &mut layer_source,
            WorldRenderContext::from_request(request),
            WorldRenderSelection {
                layer_filter: WorldLayerFilter::Include {
                    layers: &single_layer,
                    include_layerless: false,
                },
                object_filter: WorldObjectFilter::All,
                layered_image_part_filter: LayeredImagePartFilter::All,
                pass_load: WorldPassLoad::ClearTransparent,
            },
            &[],
        )?;
        renderer.composite_tinted_offscreen_over_offscreen(
            target,
            &layer_source.view,
            optical_role_debug_color(layer.role),
        )?;
    }
    Ok(())
}

pub(super) fn render_layer_mask_visual_source(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    if request
        .visual_source_flags_2d
        .is_some_and(|flags| !flags.layer_mask_generated)
    {
        // Missing/fallback visual source: draw the canonical fallback color instead of pretending a buffer exists.
        return renderer.clear_offscreen_to_color(
            target,
            color_to_wgpu(crate::renderer::service::fallback_color_for(
                amigo_render_api::VisualSourceKind2d::LayerMask,
            )),
        );
    }
    renderer.clear_offscreen_to_color(target, wgpu::Color::BLACK)?;
    let Some(layers) = camera_capture_layers(request) else {
        // Missing/fallback visual source: draw the canonical fallback color instead of pretending a buffer exists.
        return renderer.clear_offscreen_to_color(
            target,
            color_to_wgpu(crate::renderer::service::fallback_color_for(
                amigo_render_api::VisualSourceKind2d::LayerMask,
            )),
        );
    };
    let layer_count = layers.len().max(1) as f32;
    for (index, layer) in layers.iter().enumerate() {
        let single_layer = BTreeSet::from([layer.layer_id.clone()]);
        let mut layer_source = super::offscreen_ops::compatible_offscreen_target(
            target,
            "amigo-debug-layer-mask-source",
        );
        world::execute_world_to_offscreen(
            renderer,
            &mut layer_source,
            WorldRenderContext::from_request(request),
            WorldRenderSelection {
                layer_filter: WorldLayerFilter::Include {
                    layers: &single_layer,
                    include_layerless: false,
                },
                object_filter: WorldObjectFilter::All,
                layered_image_part_filter: LayeredImagePartFilter::All,
                pass_load: WorldPassLoad::ClearTransparent,
            },
            &[],
        )?;
        let shade = ((index as f32 + 1.0) / layer_count).clamp(0.12, 1.0);
        renderer.composite_tinted_offscreen_over_offscreen(
            target,
            &layer_source.view,
            ColorRgba::new(shade, shade, shade, 1.0),
        )?;
    }
    Ok(())
}

fn render_visual_source_debug_view(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    source: &wgpu::TextureView,
    visual_sources: Option<&crate::renderer::service::WgpuCameraVisualSources2d>,
    kind: amigo_render_api::VisualSourceKind2d,
    _host_id: &amigo_composite_plugin::PostFxHost2dId,
    _effect_id: &amigo_composite_plugin::PostFx2dId,
    _feature_id: &str,
) -> AmigoResult<()> {
    let Some(runtime) = visual_sources.and_then(|sources| sources.get(kind)) else {
        // Missing/fallback visual source: draw the canonical fallback color instead of pretending a buffer exists.
        return renderer.clear_offscreen_to_color(
            target,
            color_to_wgpu(crate::renderer::service::fallback_color_for(kind)),
        );
    };

    match runtime.runtime_kind {
        crate::renderer::service::WgpuVisualSourceRuntimeKind2d::AssetTexture => {
            crate::renderer::service::post_fx::wet_reflections::render_texture_asset_debug(
                renderer,
                request.assets,
                target,
                &runtime.source.id.0,
                [
                    runtime.fallback_color.r,
                    runtime.fallback_color.g,
                    runtime.fallback_color.b,
                    runtime.fallback_color.a,
                ],
                "amigo-debug-visual-source-asset",
            )
        }
        crate::renderer::service::WgpuVisualSourceRuntimeKind2d::WorldColor => {
            renderer.copy_offscreen_to_offscreen(target, source)
        }
        crate::renderer::service::WgpuVisualSourceRuntimeKind2d::WorldDepth => {
            // Missing/fallback visual source: draw the canonical fallback color instead of pretending a buffer exists.
            renderer.clear_offscreen_to_color(target, color_to_wgpu(runtime.fallback_color))
        }
        crate::renderer::service::WgpuVisualSourceRuntimeKind2d::MissingFallback => {
            // Missing/fallback visual source: draw the canonical fallback color instead of pretending a buffer exists.
            renderer.clear_offscreen_to_color(target, color_to_wgpu(runtime.fallback_color))
        }
        runtime_kind if runtime_kind.is_real_target() => {
            copy_visual_source_target_or_fallback(renderer, target, kind).map(|_| ())
        }
        _ => renderer.clear_offscreen_to_color(target, color_to_wgpu(runtime.fallback_color)),
    }
}

fn copy_visual_source_target_or_fallback(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    kind: amigo_render_api::VisualSourceKind2d,
) -> AmigoResult<bool> {
    let source_view = renderer
        .visual_source_targets_2d
        .get(kind)
        .map(|source| source.view.clone());
    if let Some(source_view) = source_view {
        renderer.copy_offscreen_to_offscreen(target, &source_view)?;
        return Ok(true);
    }
    // Missing/fallback visual source: draw the canonical fallback color instead of pretending a buffer exists.
    renderer.clear_offscreen_to_color(
        target,
        color_to_wgpu(crate::renderer::service::fallback_color_for(kind)),
    )?;
    Ok(false)
}

fn copy_layer_roles_target_or_fallback(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<bool> {
    let source_view = renderer
        .visual_source_targets_2d
        .layer_roles
        .as_ref()
        .map(|source| source.view.clone());
    if let Some(source_view) = source_view {
        renderer.copy_offscreen_to_offscreen(target, &source_view)?;
        return Ok(true);
    }
    Ok(false)
}

fn camera_debug_feature_rank(feature: &str) -> Option<u8> {
    amigo_render_api::PostFxRenderDescriptor::for_kind(feature)
        .and_then(|descriptor| descriptor.debug_policy.camera_debug_rank)
}

pub(super) fn should_bypass_for_camera_debug_view(
    debug_view: &amigo_render_api::CameraDebugView2d,
    feature_id: &str,
) -> bool {
    let Some(stop) = debug_view.stop_after_feature() else {
        return false;
    };
    let Some(stop_rank) = camera_debug_feature_rank(stop) else {
        return false;
    };
    let Some(feature_rank) = camera_debug_feature_rank(feature_id) else {
        return false;
    };
    feature_rank > stop_rank
}

pub(super) fn optical_role_debug_color(role: amigo_2d_spatial::OpticalLayerRole2d) -> ColorRgba {
    match role {
        amigo_2d_spatial::OpticalLayerRole2d::WorldSurface => ColorRgba::new(0.45, 0.78, 1.0, 0.75),
        amigo_2d_spatial::OpticalLayerRole2d::SceneMedium => ColorRgba::new(0.56, 1.0, 0.62, 0.72),
        amigo_2d_spatial::OpticalLayerRole2d::ForegroundMedium => {
            ColorRgba::new(1.0, 0.72, 0.32, 0.78)
        }
        amigo_2d_spatial::OpticalLayerRole2d::LensSurface => ColorRgba::new(1.0, 0.42, 0.86, 0.78),
        amigo_2d_spatial::OpticalLayerRole2d::Overlay => ColorRgba::new(1.0, 1.0, 0.35, 0.82),
        amigo_2d_spatial::OpticalLayerRole2d::Debug => ColorRgba::new(1.0, 0.22, 0.22, 0.85),
    }
}

#[cfg(test)]
pub(super) fn optical_debug_missing_color(kind: amigo_render_api::VisualSourceKind2d) -> ColorRgba {
    crate::renderer::service::fallback_color_for(kind)
}

fn color_to_wgpu(color: ColorRgba) -> wgpu::Color {
    wgpu::Color {
        r: color.r as f64,
        g: color.g as f64,
        b: color.b as f64,
        a: color.a as f64,
    }
}
