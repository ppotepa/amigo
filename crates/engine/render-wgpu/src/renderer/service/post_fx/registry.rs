use amigo_2d_post_fx::{PostFx2d, PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d};
use amigo_core::{AmigoError, AmigoResult};
use amigo_math::ColorRgba;
use amigo_render_api::RenderFeatureId;

use crate::{
    WgpuOffscreenTarget,
    renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer},
};

pub(crate) fn execute_screen_space_post_fx(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    host_id: &PostFxHost2dId,
    effect_id: &PostFx2dId,
    scope: &PostFxScope2d,
    pipeline: PostFxPipelineKind,
    feature_id: &RenderFeatureId,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let supported_scope_and_pipeline = matches!(
        (scope, pipeline),
        (PostFxScope2d::Frame, PostFxPipelineKind::FrameGraph)
            | (
                PostFxScope2d::DrawLayer { .. },
                PostFxPipelineKind::OffscreenDrawLayer
            )
            | (
                PostFxScope2d::SceneObjectPixels { .. },
                PostFxPipelineKind::OffscreenObject
            )
            | (
                PostFxScope2d::GroupSubtree { .. },
                PostFxPipelineKind::OffscreenGroup
            )
            | (
                PostFxScope2d::SourceImage { .. },
                PostFxPipelineKind::CachedImage
            )
            | (
                PostFxScope2d::ImagePart { .. },
                PostFxPipelineKind::CachedImage
            )
    );
    if !supported_scope_and_pipeline {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let Some(effect) = request
        .post_fx_stacks
        .iter()
        .find(|stack| &stack.host_id == host_id)
        .and_then(|stack| stack.effects.iter().find(|effect| &effect.id == effect_id))
        .map(|instance| instance.effect.clone())
    else {
        return Err(AmigoError::Message(format!(
            "post-fx effect {} from host {} is missing for feature {}",
            effect_id, host_id, feature_id
        )));
    };

    if effect.kind() != feature_id.as_str() {
        return Err(AmigoError::Message(format!(
            "post-fx feature mismatch: graph={} stack={}",
            feature_id,
            effect.kind()
        )));
    }

    match effect {
        PostFx2d::CameraExposure(effect) => {
            super::camera_exposure::execute_camera_exposure(renderer, effect, input_view, output)
        }
        PostFx2d::CameraOptics(effect) => {
            let response = (effect.flare_strength + effect.lens_bloom + effect.halation_bias)
                .clamp(0.0, 3.0)
                / 3.0;
            let normal_view = renderer
                .visual_source_targets_2d
                .scene_normal
                .as_ref()
                .map(|target| target.view.clone());
            let wetness_view = renderer
                .visual_source_targets_2d
                .scene_wetness
                .as_ref()
                .map(|target| target.view.clone());
            let highlight_view = renderer
                .visual_source_targets_2d
                .scene_highlight
                .as_ref()
                .map(|target| target.view.clone());
            let emissive_view = renderer
                .visual_source_targets_2d
                .scene_emissive
                .as_ref()
                .map(|target| target.view.clone());
            super::camera_optics::execute_camera_optics(
                renderer,
                effect,
                input_view,
                normal_view.as_ref(),
                wetness_view.as_ref(),
                highlight_view.as_ref(),
                emissive_view.as_ref(),
                output,
            )?;
            composite_camera_visual_source_response(renderer, output, response)
        }
        PostFx2d::ColorQuantize(effect) => {
            super::color_quantize::execute_color_quantize(renderer, effect, input_view, output)
        }
        PostFx2d::ColorRamp(effect) => {
            super::color_quantize::execute_color_ramp(renderer, effect, input_view, output)
        }
        PostFx2d::Crt(crt) => super::crt::execute_crt(renderer, crt, input_view, output),
        PostFx2d::Downscale(effect) => {
            super::downscale::execute_downscale(renderer, effect, input_view, output)
        }
        PostFx2d::DirtyBloom(bloom) => {
            super::dirty_bloom::execute_dirty_bloom(renderer, bloom, input_view, output)
        }
        PostFx2d::FilmEmulsion(effect) => {
            let response = (effect.shoulder + effect.push_pull.max(0.0) + effect.opacity)
                .clamp(0.0, 3.0)
                / 4.0;
            let normal_view = renderer
                .visual_source_targets_2d
                .scene_normal
                .as_ref()
                .map(|target| target.view.clone());
            let wetness_view = renderer
                .visual_source_targets_2d
                .scene_wetness
                .as_ref()
                .map(|target| target.view.clone());
            let highlight_view = renderer
                .visual_source_targets_2d
                .scene_highlight
                .as_ref()
                .map(|target| target.view.clone());
            let emissive_view = renderer
                .visual_source_targets_2d
                .scene_emissive
                .as_ref()
                .map(|target| target.view.clone());
            super::film_emulsion::execute_film_emulsion(
                renderer,
                effect,
                input_view,
                normal_view.as_ref(),
                wetness_view.as_ref(),
                highlight_view.as_ref(),
                emissive_view.as_ref(),
                output,
            )?;
            composite_camera_visual_source_response(renderer, output, response)
        }
        PostFx2d::FilmNoise(noise) => {
            super::film_noise::execute_film_noise(renderer, noise, input_view, output)
        }
        PostFx2d::FocusBlur(effect) => {
            super::focus_blur::execute_focus_blur(renderer, request, effect, input_view, output)
        }
        PostFx2d::LensDroplets(lens) => {
            super::lens_droplets::execute_lens_droplets(renderer, lens, input_view, output)
        }
        PostFx2d::RainGlass(rain) => {
            let response = (rain.scene_light_response + rain.opacity).clamp(0.0, 2.0) / 5.0;
            super::rain_glass::execute_rain_glass(
                renderer, request, host_id, effect_id, rain, input_view, output,
            )?;
            composite_camera_visual_source_response(renderer, output, response)
        }
        PostFx2d::ScanOutput(effect) => {
            super::scan_output::execute_scan_output(renderer, effect, input_view, output)
        }
        PostFx2d::ShutterBlur(effect) => super::shutter_blur::execute_shutter_blur(
            renderer, host_id, effect_id, effect, input_view, output,
        ),
        PostFx2d::WetReflections(wet) => super::wet_reflections::execute_wet_reflections(
            renderer, request, wet, input_view, output,
        ),
        PostFx2d::Blur(_) | PostFx2d::EmbossEdges(_) => {
            renderer.copy_offscreen_to_offscreen(output, input_view)
        }
    }
}

fn composite_camera_visual_source_response(
    renderer: &mut WgpuSceneRenderer,
    output: &mut WgpuOffscreenTarget,
    strength: f32,
) -> AmigoResult<()> {
    // Transitional fallback only. CameraOptics, FilmEmulsion and RainGlass sample
    // visual source buffers directly; this keeps older neutral paths visible
    // without becoming the primary optical response.
    let strength = strength.clamp(0.0, 0.12);
    if strength <= 0.0 {
        return Ok(());
    }
    let highlight_view = renderer
        .visual_source_targets_2d
        .scene_highlight
        .as_ref()
        .map(|target| target.view.clone());
    if let Some(view) = highlight_view {
        renderer.composite_tinted_offscreen_over_offscreen(
            output,
            &view,
            ColorRgba::new(strength, strength * 0.85, strength * 0.45, strength),
        )?;
    }
    let emissive_view = renderer
        .visual_source_targets_2d
        .scene_emissive
        .as_ref()
        .map(|target| target.view.clone());
    if let Some(view) = emissive_view {
        renderer.composite_tinted_offscreen_over_offscreen(
            output,
            &view,
            ColorRgba::new(strength * 0.85, strength * 0.48, strength * 0.24, strength),
        )?;
    }
    Ok(())
}
