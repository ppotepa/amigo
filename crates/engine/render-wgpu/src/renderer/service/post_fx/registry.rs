use amigo_2d_post_fx::{PostFx2d, PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d};
use amigo_core::{AmigoError, AmigoResult};
use amigo_render_api::RenderFeatureId;

use crate::{
    renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer},
    WgpuOffscreenTarget,
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
    if !matches!(pipeline, PostFxPipelineKind::FrameGraph) {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }
    if !matches!(scope, PostFxScope2d::Frame) {
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
        PostFx2d::FilmNoise(noise) => {
            super::film_noise::execute_film_noise(renderer, noise, input_view, output)
        }
        PostFx2d::LensDroplets(lens) => {
            super::lens_droplets::execute_lens_droplets(renderer, lens, input_view, output)
        }
        PostFx2d::RainGlass(rain) => {
            super::rain_glass::execute_rain_glass(renderer, request, rain, input_view, output)
        }
        PostFx2d::ShutterBlur(effect) => {
            super::shutter_blur::execute_shutter_blur(renderer, effect, input_view, output)
        }
        PostFx2d::WetReflections(wet) => super::wet_reflections::execute_wet_reflections(
            renderer, request, wet, input_view, output,
        ),
        PostFx2d::Blur(_) | PostFx2d::EmbossEdges(_) => {
            renderer.copy_offscreen_to_offscreen(output, input_view)
        }
    }
}
