#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraQualityProfile2d {
    Preview,
    #[default]
    Gameplay,
    Cinematic,
    Debug,
}

impl CameraQualityProfile2d {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "preview" => Self::Preview,
            "cinematic" | "cinema" => Self::Cinematic,
            "debug" => Self::Debug,
            _ => Self::Gameplay,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preview => "preview",
            Self::Gameplay => "gameplay",
            Self::Cinematic => "cinematic",
            Self::Debug => "debug",
        }
    }

    pub fn settings(self) -> CameraQualitySettings2d {
        match self {
            Self::Preview => CameraQualitySettings2d {
                dof_sample_scale: 0.5,
                rain_glass_resolution_scale: 0.65,
                blur_pass_scale: 0.65,
                highlight_bokeh_scale: 0.8,
                debug_buffers: false,
                generate_visual_source_buffers: false,
                generate_motion_debug_source: false,
                generate_layer_mask_debug_source: true,
                visual_source_buffer_quality: CameraVisualSourceBufferQuality2d::Skip,
                motion_source_quality: CameraVisualSourceBufferQuality2d::Skip,
                layer_mask_quality: CameraVisualSourceBufferQuality2d::Half,
            },
            Self::Gameplay => CameraQualitySettings2d::default(),
            Self::Cinematic => CameraQualitySettings2d {
                dof_sample_scale: 1.25,
                rain_glass_resolution_scale: 1.0,
                blur_pass_scale: 1.0,
                highlight_bokeh_scale: 1.15,
                debug_buffers: false,
                generate_visual_source_buffers: true,
                generate_motion_debug_source: false,
                generate_layer_mask_debug_source: true,
                visual_source_buffer_quality: CameraVisualSourceBufferQuality2d::Full,
                motion_source_quality: CameraVisualSourceBufferQuality2d::Half,
                layer_mask_quality: CameraVisualSourceBufferQuality2d::Full,
            },
            Self::Debug => CameraQualitySettings2d {
                dof_sample_scale: 1.0,
                rain_glass_resolution_scale: 1.0,
                blur_pass_scale: 1.0,
                highlight_bokeh_scale: 1.0,
                debug_buffers: true,
                generate_visual_source_buffers: true,
                generate_motion_debug_source: true,
                generate_layer_mask_debug_source: true,
                visual_source_buffer_quality: CameraVisualSourceBufferQuality2d::Full,
                motion_source_quality: CameraVisualSourceBufferQuality2d::Full,
                layer_mask_quality: CameraVisualSourceBufferQuality2d::Full,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraQualitySettings2d {
    pub dof_sample_scale: f32,
    pub rain_glass_resolution_scale: f32,
    pub blur_pass_scale: f32,
    pub highlight_bokeh_scale: f32,
    pub debug_buffers: bool,
    pub generate_visual_source_buffers: bool,
    pub generate_motion_debug_source: bool,
    pub generate_layer_mask_debug_source: bool,
    pub visual_source_buffer_quality: CameraVisualSourceBufferQuality2d,
    pub motion_source_quality: CameraVisualSourceBufferQuality2d,
    pub layer_mask_quality: CameraVisualSourceBufferQuality2d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraVisualSourceBufferQuality2d {
    Skip,
    Half,
    Full,
}

impl CameraVisualSourceBufferQuality2d {
    pub fn should_generate(self) -> bool {
        !matches!(self, Self::Skip)
    }

    pub fn resolution_scale(self) -> f32 {
        match self {
            Self::Skip => 0.0,
            Self::Half => 0.5,
            Self::Full => 1.0,
        }
    }
}

impl Default for CameraQualitySettings2d {
    fn default() -> Self {
        Self {
            dof_sample_scale: 1.0,
            rain_glass_resolution_scale: 1.0,
            blur_pass_scale: 1.0,
            highlight_bokeh_scale: 1.0,
            debug_buffers: false,
            generate_visual_source_buffers: false,
            generate_motion_debug_source: false,
            generate_layer_mask_debug_source: true,
            visual_source_buffer_quality: CameraVisualSourceBufferQuality2d::Skip,
            motion_source_quality: CameraVisualSourceBufferQuality2d::Skip,
            layer_mask_quality: CameraVisualSourceBufferQuality2d::Full,
        }
    }
}
