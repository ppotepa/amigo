#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilmGrainProfile2d {
    pub luma_amount: f32,
    pub chroma_amount: f32,
    pub shadow_amount: f32,
    pub midtone_amount: f32,
    pub highlight_amount: f32,
    pub highlight_suppression: f32,
    pub fine_grain_px: f32,
    pub medium_grain_px: f32,
    pub coarse_grain_px: f32,
    pub clumpiness: f32,
    pub softness: f32,
    pub underexposure_boost: f32,
    pub push_process_boost: f32,
    pub density_pivot: f32,
    pub channel_balance: [f32; 3],
    pub temporal_jitter: f32,
    pub regenerate_per_frame: bool,
}

impl FilmGrainProfile2d {
    pub const fn clean_digital() -> Self {
        Self {
            luma_amount: 0.08,
            chroma_amount: 0.01,
            shadow_amount: 0.08,
            midtone_amount: 0.10,
            highlight_amount: 0.03,
            highlight_suppression: 0.80,
            fine_grain_px: 0.75,
            medium_grain_px: 1.4,
            coarse_grain_px: 2.8,
            clumpiness: 0.06,
            softness: 0.55,
            underexposure_boost: 0.08,
            push_process_boost: 0.04,
            density_pivot: 0.44,
            channel_balance: [1.0, 1.0, 1.0],
            temporal_jitter: 1.0,
            regenerate_per_frame: true,
        }
    }

    pub const fn modern_color_negative() -> Self {
        Self {
            luma_amount: 0.42,
            chroma_amount: 0.12,
            shadow_amount: 0.36,
            midtone_amount: 0.48,
            highlight_amount: 0.14,
            highlight_suppression: 0.58,
            fine_grain_px: 1.0,
            medium_grain_px: 2.4,
            coarse_grain_px: 5.6,
            clumpiness: 0.24,
            softness: 0.46,
            underexposure_boost: 0.35,
            push_process_boost: 0.28,
            density_pivot: 0.42,
            channel_balance: [1.04, 0.94, 1.12],
            temporal_jitter: 1.0,
            regenerate_per_frame: true,
        }
    }

    pub const fn fast_color_negative() -> Self {
        Self {
            luma_amount: 0.54,
            chroma_amount: 0.18,
            shadow_amount: 0.52,
            midtone_amount: 0.58,
            highlight_amount: 0.18,
            highlight_suppression: 0.50,
            fine_grain_px: 1.1,
            medium_grain_px: 3.0,
            coarse_grain_px: 7.0,
            clumpiness: 0.34,
            softness: 0.42,
            underexposure_boost: 0.55,
            push_process_boost: 0.48,
            density_pivot: 0.40,
            channel_balance: [1.06, 0.92, 1.16],
            temporal_jitter: 1.0,
            regenerate_per_frame: true,
        }
    }

    pub const fn bw_silver_pushed() -> Self {
        Self {
            luma_amount: 0.78,
            chroma_amount: 0.0,
            shadow_amount: 0.72,
            midtone_amount: 0.78,
            highlight_amount: 0.28,
            highlight_suppression: 0.42,
            fine_grain_px: 1.2,
            medium_grain_px: 3.4,
            coarse_grain_px: 8.0,
            clumpiness: 0.62,
            softness: 0.18,
            underexposure_boost: 0.78,
            push_process_boost: 0.82,
            density_pivot: 0.38,
            channel_balance: [1.0, 1.0, 1.0],
            temporal_jitter: 1.0,
            regenerate_per_frame: true,
        }
    }

    pub const fn fine_reversal() -> Self {
        Self {
            luma_amount: 0.24,
            chroma_amount: 0.06,
            shadow_amount: 0.22,
            midtone_amount: 0.30,
            highlight_amount: 0.08,
            highlight_suppression: 0.70,
            fine_grain_px: 0.8,
            medium_grain_px: 1.8,
            coarse_grain_px: 3.6,
            clumpiness: 0.14,
            softness: 0.30,
            underexposure_boost: 0.22,
            push_process_boost: 0.20,
            density_pivot: 0.46,
            channel_balance: [0.98, 0.98, 1.06],
            temporal_jitter: 1.0,
            regenerate_per_frame: true,
        }
    }

    pub const fn dirty_scan() -> Self {
        Self {
            luma_amount: 0.62,
            chroma_amount: 0.22,
            shadow_amount: 0.64,
            midtone_amount: 0.62,
            highlight_amount: 0.22,
            highlight_suppression: 0.36,
            fine_grain_px: 1.0,
            medium_grain_px: 3.2,
            coarse_grain_px: 7.6,
            clumpiness: 0.52,
            softness: 0.36,
            underexposure_boost: 0.68,
            push_process_boost: 0.62,
            density_pivot: 0.39,
            channel_balance: [1.12, 0.90, 1.22],
            temporal_jitter: 1.0,
            regenerate_per_frame: true,
        }
    }
}
