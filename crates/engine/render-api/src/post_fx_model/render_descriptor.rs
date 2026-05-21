use crate::PostFx2d;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostFxRenderInput {
    SourceColor,
    SceneNormal,
    SceneWetness,
    SceneHighlight,
    SceneEmissive,
    SceneMotion,
    FrameRequest,
    HostEffectIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostFxRenderOutput {
    ReplaceTarget,
    CompositeVisualSourceResponse,
    ReplayScopedLayers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostFxCachedImagePolicy {
    Unsupported,
    PassthroughCopy,
    RasterEffect,
    RasterEffectWithBoundsExpansion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PostFxDebugPolicy {
    pub camera_debug_rank: Option<u8>,
    pub supports_depth_debug_view: bool,
}

impl PostFxDebugPolicy {
    pub const fn none() -> Self {
        Self {
            camera_debug_rank: None,
            supports_depth_debug_view: false,
        }
    }

    pub const fn camera_chain(rank: u8) -> Self {
        Self {
            camera_debug_rank: Some(rank),
            supports_depth_debug_view: false,
        }
    }

    pub const fn focus_blur() -> Self {
        Self {
            camera_debug_rank: Some(40),
            supports_depth_debug_view: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PostFxRenderDescriptor {
    pub feature_id: &'static str,
    pub executor_id: &'static str,
    pub required_inputs: &'static [PostFxRenderInput],
    pub output: PostFxRenderOutput,
    pub cached_image_policy: PostFxCachedImagePolicy,
    pub frame_graph_compatible: bool,
    pub debug_policy: PostFxDebugPolicy,
}

impl PostFxRenderDescriptor {
    pub fn for_effect(effect: &PostFx2d) -> Self {
        match effect {
            PostFx2d::Blur(_) => blur_descriptor(),
            PostFx2d::CameraExposure(_) => camera_exposure_descriptor(),
            PostFx2d::CameraOptics(_) => camera_optics_descriptor(),
            PostFx2d::ColorQuantize(_) => color_quantize_descriptor(),
            PostFx2d::ColorRamp(_) => color_ramp_descriptor(),
            PostFx2d::Crt(_) => crt_descriptor(),
            PostFx2d::Downscale(_) => downscale_descriptor(),
            PostFx2d::DirtyBloom(_) => dirty_bloom_descriptor(),
            PostFx2d::EmbossEdges(_) => emboss_edges_descriptor(),
            PostFx2d::FilmEmulsion(_) => film_emulsion_descriptor(),
            PostFx2d::FilmNoise(_) => film_noise_descriptor(),
            PostFx2d::FocusBlur(_) => focus_blur_descriptor(),
            PostFx2d::LensDroplets(_) => lens_droplets_descriptor(),
            PostFx2d::RainGlass(_) => rain_glass_descriptor(),
            PostFx2d::ScanOutput(_) => scan_output_descriptor(),
            PostFx2d::ShutterBlur(_) => shutter_blur_descriptor(),
            PostFx2d::WetReflections(_) => wet_reflections_descriptor(),
        }
    }

    pub fn for_kind(kind: &str) -> Option<Self> {
        match kind {
            "blur" => Some(blur_descriptor()),
            "camera_exposure" => Some(camera_exposure_descriptor()),
            "camera_optics" => Some(camera_optics_descriptor()),
            "color_quantize" => Some(color_quantize_descriptor()),
            "color_ramp" => Some(color_ramp_descriptor()),
            "crt" => Some(crt_descriptor()),
            "downscale" => Some(downscale_descriptor()),
            "dirty_bloom" => Some(dirty_bloom_descriptor()),
            "embossed_edges" => Some(emboss_edges_descriptor()),
            "film_emulsion" => Some(film_emulsion_descriptor()),
            "film_noise" => Some(film_noise_descriptor()),
            "focus_blur" => Some(focus_blur_descriptor()),
            "lens_droplets" => Some(lens_droplets_descriptor()),
            "rain_glass" => Some(rain_glass_descriptor()),
            "scan_output" => Some(scan_output_descriptor()),
            "shutter_blur" => Some(shutter_blur_descriptor()),
            "wet_reflections" => Some(wet_reflections_descriptor()),
            _ => None,
        }
    }
}

impl PostFx2d {
    pub fn render_descriptor(&self) -> PostFxRenderDescriptor {
        PostFxRenderDescriptor::for_effect(self)
    }
}

const SOURCE_ONLY: &[PostFxRenderInput] = &[PostFxRenderInput::SourceColor];
const CAMERA_OPTICS_INPUTS: &[PostFxRenderInput] = &[
    PostFxRenderInput::SourceColor,
    PostFxRenderInput::SceneNormal,
    PostFxRenderInput::SceneWetness,
    PostFxRenderInput::SceneHighlight,
    PostFxRenderInput::SceneEmissive,
];
const REQUEST_ONLY_INPUTS: &[PostFxRenderInput] = &[
    PostFxRenderInput::SourceColor,
    PostFxRenderInput::FrameRequest,
];
const HOST_EFFECT_INPUTS: &[PostFxRenderInput] = &[
    PostFxRenderInput::SourceColor,
    PostFxRenderInput::HostEffectIdentity,
];
const REQUEST_AND_HOST_EFFECT_INPUTS: &[PostFxRenderInput] = &[
    PostFxRenderInput::SourceColor,
    PostFxRenderInput::FrameRequest,
    PostFxRenderInput::HostEffectIdentity,
];

const fn descriptor(
    feature_id: &'static str,
    executor_id: &'static str,
    required_inputs: &'static [PostFxRenderInput],
    output: PostFxRenderOutput,
    cached_image_policy: PostFxCachedImagePolicy,
    frame_graph_compatible: bool,
    debug_policy: PostFxDebugPolicy,
) -> PostFxRenderDescriptor {
    PostFxRenderDescriptor {
        feature_id,
        executor_id,
        required_inputs,
        output,
        cached_image_policy,
        frame_graph_compatible,
        debug_policy,
    }
}

const fn blur_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "blur",
        "screen_space.blur",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::RasterEffectWithBoundsExpansion,
        false,
        PostFxDebugPolicy::none(),
    )
}

const fn camera_exposure_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "camera_exposure",
        "screen_space.camera_exposure",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::camera_chain(10),
    )
}

const fn camera_optics_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "camera_optics",
        "screen_space.camera_optics",
        CAMERA_OPTICS_INPUTS,
        PostFxRenderOutput::CompositeVisualSourceResponse,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::camera_chain(30),
    )
}

const fn color_quantize_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "color_quantize",
        "screen_space.color_quantize",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::none(),
    )
}

const fn color_ramp_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "color_ramp",
        "screen_space.color_ramp",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::camera_chain(70),
    )
}

const fn crt_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "crt",
        "screen_space.crt",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::none(),
    )
}

const fn downscale_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "downscale",
        "screen_space.downscale",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::none(),
    )
}

const fn dirty_bloom_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "dirty_bloom",
        "screen_space.dirty_bloom",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        false,
        PostFxDebugPolicy::none(),
    )
}

const fn emboss_edges_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "embossed_edges",
        "screen_space.embossed_edges",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::RasterEffect,
        false,
        PostFxDebugPolicy::none(),
    )
}

const fn film_emulsion_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "film_emulsion",
        "screen_space.film_emulsion",
        CAMERA_OPTICS_INPUTS,
        PostFxRenderOutput::CompositeVisualSourceResponse,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::camera_chain(60),
    )
}

const fn film_noise_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "film_noise",
        "screen_space.film_noise",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::none(),
    )
}

const fn focus_blur_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "focus_blur",
        "screen_space.focus_blur",
        REQUEST_ONLY_INPUTS,
        PostFxRenderOutput::ReplayScopedLayers,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::focus_blur(),
    )
}

const fn lens_droplets_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "lens_droplets",
        "screen_space.lens_droplets",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        false,
        PostFxDebugPolicy::none(),
    )
}

const fn rain_glass_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "rain_glass",
        "screen_space.rain_glass",
        REQUEST_AND_HOST_EFFECT_INPUTS,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::camera_chain(50),
    )
}

const fn scan_output_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "scan_output",
        "screen_space.scan_output",
        SOURCE_ONLY,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::camera_chain(80),
    )
}

const fn shutter_blur_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "shutter_blur",
        "screen_space.shutter_blur",
        HOST_EFFECT_INPUTS,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        true,
        PostFxDebugPolicy::camera_chain(20),
    )
}

const fn wet_reflections_descriptor() -> PostFxRenderDescriptor {
    descriptor(
        "wet_reflections",
        "screen_space.wet_reflections",
        REQUEST_ONLY_INPUTS,
        PostFxRenderOutput::ReplaceTarget,
        PostFxCachedImagePolicy::PassthroughCopy,
        false,
        PostFxDebugPolicy::none(),
    )
}
