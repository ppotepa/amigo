use amigo_2d_post_fx::PostFx2d;
use amigo_core::{AmigoError, AmigoResult};
use amigo_render_api::RenderFeatureId;

use crate::{
    WgpuOffscreenTarget,
    renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer},
};

pub(crate) fn execute_screen_space_post_fx(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    feature_id: &RenderFeatureId,
    effect_index: usize,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let Some(stack) = request.post_fx_stack else {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    };

    let Some(effect) = stack.effects.get(effect_index).cloned() else {
        return Err(AmigoError::Message(format!(
            "post-fx effect index {} is missing for feature {}",
            effect_index, feature_id
        )));
    };

    if effect.clone().kind() != feature_id.as_str() {
        return Err(AmigoError::Message(format!(
            "post-fx feature mismatch: graph={} stack={}",
            feature_id,
            effect.kind()
        )));
    }

    match effect {
        PostFx2d::LensDroplets(lens) => {
            super::lens_droplets::execute_lens_droplets(renderer, lens, input_view, output)
        }
        PostFx2d::WetReflections(wet) => {
            super::wet_reflections::execute_wet_reflections(renderer, request, wet, input_view, output)
        }
        PostFx2d::Blur(_) | PostFx2d::EmbossEdges(_) => {
            renderer.copy_offscreen_to_offscreen(output, input_view)
        }
    }
}

