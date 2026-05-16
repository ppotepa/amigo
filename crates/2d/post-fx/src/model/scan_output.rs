use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScanOutput2d {
    pub iso: f32,
    pub flicker: f32,
    pub vignette: f32,
    pub print_fade: f32,
    pub dust: f32,
    pub scratches: f32,
    pub gate_weave: f32,
    pub scan_softness: f32,
    pub opacity: f32,
    pub seed: u32,
    pub grain_chroma: f32,
    pub grain_luma: f32,
    pub shadow_grain: f32,
    pub midtone_grain: f32,
    pub highlight_grain: f32,
    pub highlight_suppression: f32,
    pub fine_grain_px: f32,
    pub medium_grain_px: f32,
    pub coarse_grain_px: f32,
    pub clumpiness: f32,
    pub grain_softness: f32,
    pub underexposure_grain_boost: f32,
    pub push_process_boost: f32,
    pub density_pivot: f32,
    pub channel_balance: [f32; 3],
    pub temporal_jitter: f32,
    pub grain_regenerate_per_frame: bool,
}

impl Default for ScanOutput2d {
    fn default() -> Self {
        Self {
            iso: 800.0,
            flicker: 0.12,
            vignette: 0.08,
            print_fade: 0.08,
            dust: 0.0,
            scratches: 0.0,
            gate_weave: 0.0,
            scan_softness: 0.0,
            opacity: 0.35,
            seed: 1337,
            grain_chroma: 0.04,
            grain_luma: 0.42,
            shadow_grain: 0.36,
            midtone_grain: 0.48,
            highlight_grain: 0.14,
            highlight_suppression: 0.58,
            fine_grain_px: 1.0,
            medium_grain_px: 2.4,
            coarse_grain_px: 5.6,
            clumpiness: 0.24,
            grain_softness: 0.46,
            underexposure_grain_boost: 0.35,
            push_process_boost: 0.28,
            density_pivot: 0.42,
            channel_balance: [1.04, 0.94, 1.12],
            temporal_jitter: 1.0,
            grain_regenerate_per_frame: true,
        }
    }
}

impl ScanOutput2d {
    pub fn normalized(mut self) -> Self {
        self.iso = finite_or(self.iso, 800.0).clamp(50.0, 25600.0);
        self.flicker = finite_or(self.flicker, 0.12).clamp(0.0, 1.0);
        self.vignette = finite_or(self.vignette, 0.08).clamp(0.0, 1.0);
        self.print_fade = finite_or(self.print_fade, 0.08).clamp(0.0, 1.0);
        self.dust = finite_or(self.dust, 0.0).clamp(0.0, 1.0);
        self.scratches = finite_or(self.scratches, 0.0).clamp(0.0, 1.0);
        self.gate_weave = finite_or(self.gate_weave, 0.0).clamp(0.0, 1.0);
        self.scan_softness = finite_or(self.scan_softness, 0.0).clamp(0.0, 1.0);
        self.opacity = finite_or(self.opacity, 0.35).clamp(0.0, 1.0);
        self.grain_chroma = finite_or(self.grain_chroma, 0.04).clamp(0.0, 1.0);
        self.grain_luma = finite_or(self.grain_luma, 0.42).clamp(0.0, 2.0);
        self.shadow_grain = finite_or(self.shadow_grain, 0.36).clamp(0.0, 2.0);
        self.midtone_grain = finite_or(self.midtone_grain, 0.48).clamp(0.0, 2.0);
        self.highlight_grain = finite_or(self.highlight_grain, 0.14).clamp(0.0, 2.0);
        self.highlight_suppression = finite_or(self.highlight_suppression, 0.58).clamp(0.0, 1.0);
        self.fine_grain_px = finite_or(self.fine_grain_px, 1.0).clamp(0.5, 8.0);
        self.medium_grain_px = finite_or(self.medium_grain_px, 2.4).clamp(0.5, 16.0);
        self.coarse_grain_px = finite_or(self.coarse_grain_px, 5.6).clamp(0.5, 32.0);
        self.clumpiness = finite_or(self.clumpiness, 0.24).clamp(0.0, 1.0);
        self.grain_softness = finite_or(self.grain_softness, 0.46).clamp(0.0, 1.0);
        self.underexposure_grain_boost =
            finite_or(self.underexposure_grain_boost, 0.35).clamp(0.0, 2.0);
        self.push_process_boost = finite_or(self.push_process_boost, 0.28).clamp(0.0, 2.0);
        self.density_pivot = finite_or(self.density_pivot, 0.42).clamp(0.10, 0.75);
        self.channel_balance = [
            finite_or(self.channel_balance[0], 1.04).clamp(0.0, 2.0),
            finite_or(self.channel_balance[1], 0.94).clamp(0.0, 2.0),
            finite_or(self.channel_balance[2], 1.12).clamp(0.0, 2.0),
        ];
        self.temporal_jitter = finite_or(self.temporal_jitter, 1.0).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.iso > 50.0 && self.opacity > 0.0
    }
}
