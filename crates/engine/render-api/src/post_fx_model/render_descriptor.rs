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
    pub frame_graph_enabled: bool,
    pub debug_policy: PostFxDebugPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PostFxRenderDescriptorEntry {
    pub kind: &'static str,
    pub descriptor: PostFxRenderDescriptor,
}

pub const POST_FX_RENDER_DESCRIPTOR_REGISTRY: &[PostFxRenderDescriptorEntry] = &[
    descriptor_entry("blur", blur_descriptor()),
    descriptor_entry("camera_exposure", camera_exposure_descriptor()),
    descriptor_entry("camera_optics", camera_optics_descriptor()),
    descriptor_entry("color_quantize", color_quantize_descriptor()),
    descriptor_entry("color_ramp", color_ramp_descriptor()),
    descriptor_entry("crt", crt_descriptor()),
    descriptor_entry("downscale", downscale_descriptor()),
    descriptor_entry("dirty_bloom", dirty_bloom_descriptor()),
    descriptor_entry("embossed_edges", emboss_edges_descriptor()),
    descriptor_entry("film_emulsion", film_emulsion_descriptor()),
    descriptor_entry("film_noise", film_noise_descriptor()),
    descriptor_entry("focus_blur", focus_blur_descriptor()),
    descriptor_entry("lens_droplets", lens_droplets_descriptor()),
    descriptor_entry("rain_glass", rain_glass_descriptor()),
    descriptor_entry("scan_output", scan_output_descriptor()),
    descriptor_entry("shutter_blur", shutter_blur_descriptor()),
    descriptor_entry("wet_reflections", wet_reflections_descriptor()),
];

impl PostFxRenderDescriptor {
    pub fn registry() -> &'static [PostFxRenderDescriptorEntry] {
        POST_FX_RENDER_DESCRIPTOR_REGISTRY
    }

    pub fn for_kind(kind: &str) -> Option<Self> {
        Self::registry()
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.descriptor)
    }

    pub fn requires_executor(&self) -> bool {
        !self.executor_id.is_empty()
    }
}

impl crate::PostFx2d {
    pub fn render_descriptor(&self) -> PostFxRenderDescriptor {
        PostFxRenderDescriptor::for_kind(self.kind())
            .expect("PostFx2d variants must have render descriptors")
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
    frame_graph_enabled: bool,
    debug_policy: PostFxDebugPolicy,
) -> PostFxRenderDescriptor {
    PostFxRenderDescriptor {
        feature_id,
        executor_id,
        required_inputs,
        output,
        cached_image_policy,
        frame_graph_enabled,
        debug_policy,
    }
}

const fn descriptor_entry(
    kind: &'static str,
    descriptor: PostFxRenderDescriptor,
) -> PostFxRenderDescriptorEntry {
    PostFxRenderDescriptorEntry { kind, descriptor }
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
