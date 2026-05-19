use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilmNoise2d {
    pub iso: f32,
    pub grain_size: f32,
    pub chroma_noise: f32,
    pub color_shift: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub flicker: f32,
    pub vignette: f32,
    pub toe: f32,
    pub shoulder: f32,
    pub black_lift: f32,
    pub print_fade: f32,
    pub dust: f32,
    pub scratches: f32,
    pub push_pull: f32,
    pub gate_weave: f32,
    pub scan_softness: f32,
    pub opacity: f32,
    pub seed: u32,
}

impl Default for FilmNoise2d {
    fn default() -> Self {
        Self {
            iso: 800.0,
            grain_size: 1.0,
            chroma_noise: 0.04,
            color_shift: 0.03,
            contrast: 1.0,
            saturation: 1.0,
            flicker: 0.12,
            vignette: 0.08,
            toe: 0.45,
            shoulder: 0.65,
            black_lift: 0.02,
            print_fade: 0.08,
            dust: 0.0,
            scratches: 0.0,
            push_pull: 0.0,
            gate_weave: 0.0,
            scan_softness: 0.0,
            opacity: 0.35,
            seed: 1337,
        }
    }
}

impl FilmNoise2d {
    pub fn normalized(mut self) -> Self {
        self.iso = finite_or(self.iso, 800.0).clamp(50.0, 25600.0);
        self.grain_size = finite_or(self.grain_size, 1.0).clamp(0.25, 8.0);
        self.chroma_noise = finite_or(self.chroma_noise, 0.04).clamp(0.0, 1.0);
        self.color_shift = finite_or(self.color_shift, 0.03).clamp(-1.0, 1.0);
        self.contrast = finite_or(self.contrast, 1.0).clamp(0.25, 4.0);
        self.saturation = finite_or(self.saturation, 1.0).clamp(0.0, 4.0);
        self.flicker = finite_or(self.flicker, 0.12).clamp(0.0, 1.0);
        self.vignette = finite_or(self.vignette, 0.08).clamp(0.0, 1.0);
        self.toe = finite_or(self.toe, 0.45).clamp(0.0, 1.0);
        self.shoulder = finite_or(self.shoulder, 0.65).clamp(0.0, 1.0);
        self.black_lift = finite_or(self.black_lift, 0.02).clamp(0.0, 0.4);
        self.print_fade = finite_or(self.print_fade, 0.08).clamp(0.0, 1.0);
        self.dust = finite_or(self.dust, 0.0).clamp(0.0, 1.0);
        self.scratches = finite_or(self.scratches, 0.0).clamp(0.0, 1.0);
        self.push_pull = finite_or(self.push_pull, 0.0).clamp(-4.0, 4.0);
        self.gate_weave = finite_or(self.gate_weave, 0.0).clamp(0.0, 1.0);
        self.scan_softness = finite_or(self.scan_softness, 0.0).clamp(0.0, 1.0);
        self.opacity = finite_or(self.opacity, 0.35).clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.iso > 50.0 && self.opacity > 0.0
    }
}
