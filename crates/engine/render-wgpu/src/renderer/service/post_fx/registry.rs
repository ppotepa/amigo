use amigo_core::{AmigoError, AmigoResult};
use amigo_math::ColorRgba;
use amigo_render_api::{
    CameraExposure2d, CameraOptics2d, ColorQuantize2d, ColorRamp2d, Crt2d, DirtyBloom2d,
    FilmEmulsion2d, FilmNoise2d, FocusBlur2d, PostFx2d, PostFx2dId, PostFxHost2dId,
    PostFxLensDroplets2d, PostFxPipelineKind, PostFxScope2d, PostFxWetReflections2d,
    RainGlass2d, RenderFeatureId, ScanOutput2d, ShutterBlur2d, Downscale2d,
};

use crate::{
    WgpuOffscreenTarget,
    renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer},
};

use super::{
    WgpuPostFxExecutionContext, WgpuPostFxExecutionContext as Context, WgpuPostFxExecutor,
    WgpuPostFxExecutorRegistry,
};

struct DescriptorVisualSourceViews {
    normal: Option<wgpu::TextureView>,
    wetness: Option<wgpu::TextureView>,
    highlight: Option<wgpu::TextureView>,
    emissive: Option<wgpu::TextureView>,
}

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

    let effect_kind = effect.kind();
    let descriptor = effect.render_descriptor();

    if descriptor.feature_id != feature_id.as_str() {
        return Err(AmigoError::Message(format!(
            "post-fx feature mismatch: graph={} stack={}",
            feature_id,
            effect_kind
        )));
    }

    let ctx = WgpuPostFxExecutionContext {
        request,
        host_id,
        effect_id,
        _feature_id: feature_id,
        descriptor: &descriptor,
        effect,
        input_view,
        output,
    };
    let executor = renderer
        .post_fx_executors
        .executor(ctx.descriptor.executor_id, effect_kind)?;
    executor.execute(renderer, ctx)
}

pub(crate) fn default_post_fx_executor_registry() -> WgpuPostFxExecutorRegistry {
    let mut registry = WgpuPostFxExecutorRegistry::default();
    registry.register(CameraExposureExecutor);
    registry.register(CameraOpticsExecutor);
    registry.register(ColorQuantizeExecutor);
    registry.register(ColorRampExecutor);
    registry.register(CrtExecutor);
    registry.register(DownscaleExecutor);
    registry.register(DirtyBloomExecutor);
    registry.register(FilmEmulsionExecutor);
    registry.register(FilmNoiseExecutor);
    registry.register(FocusBlurExecutor);
    registry.register(LensDropletsExecutor);
    registry.register(RainGlassExecutor);
    registry.register(ScanOutputExecutor);
    registry.register(ShutterBlurExecutor);
    registry.register(WetReflectionsExecutor);
    registry.register(CopyThroughExecutor(crate::renderer::service::POST_FX_EXECUTOR_BLUR));
    registry.register(CopyThroughExecutor(
        crate::renderer::service::POST_FX_EXECUTOR_EMBOSSED_EDGES,
    ));
    registry
}

struct CameraExposureExecutor;
struct CameraOpticsExecutor;
struct ColorQuantizeExecutor;
struct ColorRampExecutor;
struct CrtExecutor;
struct DownscaleExecutor;
struct DirtyBloomExecutor;
struct FilmEmulsionExecutor;
struct FilmNoiseExecutor;
struct FocusBlurExecutor;
struct LensDropletsExecutor;
struct RainGlassExecutor;
struct ScanOutputExecutor;
struct ShutterBlurExecutor;
struct WetReflectionsExecutor;
struct CopyThroughExecutor(&'static str);

macro_rules! effect_executor {
    ($name:ident, $executor_id:expr, $variant:path, |$renderer:ident, $ctx:ident, $effect:ident| $body:expr) => {
        impl WgpuPostFxExecutor for $name {
            fn executor_id(&self) -> &'static str {
                $executor_id
            }

            fn execute(
                &self,
                $renderer: &mut WgpuSceneRenderer,
                $ctx: Context<'_>,
            ) -> AmigoResult<()> {
                let $variant($effect) = $ctx.effect else {
                    return Err(post_fx_executor_mismatch_error(
                        $ctx.descriptor.executor_id,
                        $ctx.effect.kind(),
                    ));
                };
                $body
            }
        }
    };
}

effect_executor!(
    CameraExposureExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_CAMERA_EXPOSURE,
    PostFx2d::CameraExposure,
    |renderer, ctx, effect| execute_camera_exposure(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    CameraOpticsExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_CAMERA_OPTICS,
    PostFx2d::CameraOptics,
    |renderer, ctx, effect| execute_camera_optics(
        renderer,
        effect,
        ctx.descriptor,
        ctx.input_view,
        ctx.output,
    )
);
effect_executor!(
    ColorQuantizeExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_COLOR_QUANTIZE,
    PostFx2d::ColorQuantize,
    |renderer, ctx, effect| execute_color_quantize(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    ColorRampExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_COLOR_RAMP,
    PostFx2d::ColorRamp,
    |renderer, ctx, effect| execute_color_ramp(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    CrtExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_CRT,
    PostFx2d::Crt,
    |renderer, ctx, effect| execute_crt(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    DownscaleExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_DOWNSCALE,
    PostFx2d::Downscale,
    |renderer, ctx, effect| execute_downscale(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    DirtyBloomExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_DIRTY_BLOOM,
    PostFx2d::DirtyBloom,
    |renderer, ctx, effect| execute_dirty_bloom(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    FilmEmulsionExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_FILM_EMULSION,
    PostFx2d::FilmEmulsion,
    |renderer, ctx, effect| execute_film_emulsion(
        renderer,
        effect,
        ctx.descriptor,
        ctx.input_view,
        ctx.output,
    )
);
effect_executor!(
    FilmNoiseExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_FILM_NOISE,
    PostFx2d::FilmNoise,
    |renderer, ctx, effect| execute_film_noise(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    FocusBlurExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_FOCUS_BLUR,
    PostFx2d::FocusBlur,
    |renderer, ctx, effect| execute_focus_blur(
        renderer,
        ctx.request,
        effect,
        ctx.input_view,
        ctx.output,
    )
);
effect_executor!(
    LensDropletsExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_LENS_DROPLETS,
    PostFx2d::LensDroplets,
    |renderer, ctx, effect| execute_lens_droplets(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    RainGlassExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_RAIN_GLASS,
    PostFx2d::RainGlass,
    |renderer, ctx, effect| execute_rain_glass(
        renderer,
        ctx.request,
        ctx.host_id,
        ctx.effect_id,
        effect,
        ctx.descriptor,
        ctx.input_view,
        ctx.output,
    )
);
effect_executor!(
    ScanOutputExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_SCAN_OUTPUT,
    PostFx2d::ScanOutput,
    |renderer, ctx, effect| execute_scan_output(renderer, effect, ctx.input_view, ctx.output)
);
effect_executor!(
    ShutterBlurExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_SHUTTER_BLUR,
    PostFx2d::ShutterBlur,
    |renderer, ctx, effect| execute_shutter_blur(
        renderer,
        ctx.host_id,
        ctx.effect_id,
        effect,
        ctx.input_view,
        ctx.output,
    )
);
effect_executor!(
    WetReflectionsExecutor,
    crate::renderer::service::POST_FX_EXECUTOR_WET_REFLECTIONS,
    PostFx2d::WetReflections,
    |renderer, ctx, effect| execute_wet_reflections(
        renderer,
        ctx.request,
        effect,
        ctx.input_view,
        ctx.output,
    )
);

impl WgpuPostFxExecutor for CopyThroughExecutor {
    fn executor_id(&self) -> &'static str {
        self.0
    }

    fn execute(
        &self,
        renderer: &mut WgpuSceneRenderer,
        ctx: Context<'_>,
    ) -> AmigoResult<()> {
        renderer.copy_offscreen_to_offscreen(ctx.output, ctx.input_view)
    }
}

fn execute_camera_exposure(
    renderer: &mut WgpuSceneRenderer,
    effect: CameraExposure2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::camera_exposure::execute_camera_exposure(renderer, effect, input_view, output)
}

fn execute_camera_optics(
    renderer: &mut WgpuSceneRenderer,
    effect: CameraOptics2d,
    descriptor: &amigo_render_api::PostFxRenderDescriptor,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let visual_sources = descriptor_visual_source_views(renderer, descriptor);
    let response = (effect.glare_strength + effect.lens_bloom + effect.halation_bias)
        .clamp(0.0, 3.0)
        / 3.0;
    super::camera_optics::execute_camera_optics(
        renderer,
        effect,
        input_view,
        visual_sources.normal.as_ref(),
        visual_sources.wetness.as_ref(),
        visual_sources.highlight.as_ref(),
        visual_sources.emissive.as_ref(),
        output,
    )?;
    composite_descriptor_visual_source_response(renderer, descriptor, output, response)
}

fn execute_color_quantize(
    renderer: &mut WgpuSceneRenderer,
    effect: ColorQuantize2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::color_quantize::execute_color_quantize(renderer, effect, input_view, output)
}

fn execute_color_ramp(
    renderer: &mut WgpuSceneRenderer,
    effect: ColorRamp2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::color_quantize::execute_color_ramp(renderer, effect, input_view, output)
}

fn execute_crt(
    renderer: &mut WgpuSceneRenderer,
    effect: Crt2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::crt::execute_crt(renderer, effect, input_view, output)
}

fn execute_downscale(
    renderer: &mut WgpuSceneRenderer,
    effect: Downscale2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::downscale::execute_downscale(renderer, effect, input_view, output)
}

fn execute_dirty_bloom(
    renderer: &mut WgpuSceneRenderer,
    effect: DirtyBloom2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::dirty_bloom::execute_dirty_bloom(renderer, effect, input_view, output)
}

fn execute_film_emulsion(
    renderer: &mut WgpuSceneRenderer,
    effect: FilmEmulsion2d,
    descriptor: &amigo_render_api::PostFxRenderDescriptor,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let visual_sources = descriptor_visual_source_views(renderer, descriptor);
    let response = (effect.shoulder + effect.push_pull.max(0.0) + effect.opacity)
        .clamp(0.0, 3.0)
        / 4.0;
    super::film_emulsion::execute_film_emulsion(
        renderer,
        effect,
        input_view,
        visual_sources.normal.as_ref(),
        visual_sources.wetness.as_ref(),
        visual_sources.highlight.as_ref(),
        visual_sources.emissive.as_ref(),
        output,
    )?;
    composite_descriptor_visual_source_response(renderer, descriptor, output, response)
}

fn execute_film_noise(
    renderer: &mut WgpuSceneRenderer,
    effect: FilmNoise2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::film_noise::execute_film_noise(renderer, effect, input_view, output)
}

fn execute_focus_blur(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    effect: FocusBlur2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::focus_blur::execute_focus_blur(renderer, request, effect, input_view, output)
}

fn execute_lens_droplets(
    renderer: &mut WgpuSceneRenderer,
    effect: PostFxLensDroplets2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::lens_droplets::execute_lens_droplets(renderer, effect, input_view, output)
}

fn execute_rain_glass(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    host_id: &PostFxHost2dId,
    effect_id: &PostFx2dId,
    effect: RainGlass2d,
    descriptor: &amigo_render_api::PostFxRenderDescriptor,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let response = (effect.scene_light_response + effect.opacity).clamp(0.0, 2.0) / 5.0;
    super::rain_glass::execute_rain_glass(
        renderer, request, host_id, effect_id, effect, input_view, output,
    )?;
    composite_descriptor_visual_source_response(renderer, descriptor, output, response)
}

fn execute_scan_output(
    renderer: &mut WgpuSceneRenderer,
    effect: ScanOutput2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::scan_output::execute_scan_output(renderer, effect, input_view, output)
}

fn execute_shutter_blur(
    renderer: &mut WgpuSceneRenderer,
    host_id: &PostFxHost2dId,
    effect_id: &PostFx2dId,
    effect: ShutterBlur2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::shutter_blur::execute_shutter_blur(renderer, host_id, effect_id, effect, input_view, output)
}

fn execute_wet_reflections(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    effect: PostFxWetReflections2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    super::wet_reflections::execute_wet_reflections(renderer, request, effect, input_view, output)
}

fn descriptor_visual_source_views(
    renderer: &WgpuSceneRenderer,
    descriptor: &amigo_render_api::PostFxRenderDescriptor,
) -> DescriptorVisualSourceViews {
    let wants = |input| descriptor.required_inputs.contains(&input);
    DescriptorVisualSourceViews {
        normal: wants(amigo_render_api::PostFxRenderInput::SceneNormal).then(|| {
            renderer
                .visual_source_targets_2d
                .scene_normal
                .as_ref()
                .map(|target| target.view.clone())
        }).flatten(),
        wetness: wants(amigo_render_api::PostFxRenderInput::SceneWetness).then(|| {
            renderer
                .visual_source_targets_2d
                .scene_wetness
                .as_ref()
                .map(|target| target.view.clone())
        }).flatten(),
        highlight: wants(amigo_render_api::PostFxRenderInput::SceneHighlight).then(|| {
            renderer
                .visual_source_targets_2d
                .scene_highlight
                .as_ref()
                .map(|target| target.view.clone())
        }).flatten(),
        emissive: wants(amigo_render_api::PostFxRenderInput::SceneEmissive).then(|| {
            renderer
                .visual_source_targets_2d
                .scene_emissive
                .as_ref()
                .map(|target| target.view.clone())
        }).flatten(),
    }
}

fn composite_descriptor_visual_source_response(
    renderer: &mut WgpuSceneRenderer,
    descriptor: &amigo_render_api::PostFxRenderDescriptor,
    output: &mut WgpuOffscreenTarget,
    strength: f32,
) -> AmigoResult<()> {
    if descriptor.output != amigo_render_api::PostFxRenderOutput::CompositeVisualSourceResponse {
        return Ok(());
    }
    composite_camera_visual_source_response(renderer, output, strength)
}

fn post_fx_executor_mismatch_error(executor_id: &str, effect_kind: &str) -> AmigoError {
    AmigoError::Message(format!(
        "post-fx executor mismatch: executor={} effect={}",
        executor_id, effect_kind
    ))
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
