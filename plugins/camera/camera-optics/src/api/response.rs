#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraOpticalResponse2d {
    pub enabled: bool,
    pub intensity: f32,
    pub bloom: f32,
    pub glare: f32,
    pub ghosting: f32,
    pub streaks: f32,
    pub chromatic_smear: f32,
    pub dirt_response: f32,
    pub halation: f32,
    pub threshold: f32,
}

impl Default for CameraOpticalResponse2d {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.0,
            bloom: 0.0,
            glare: 0.0,
            ghosting: 0.0,
            streaks: 0.0,
            chromatic_smear: 0.0,
            dirt_response: 0.0,
            halation: 0.0,
            threshold: 0.0,
        }
    }
}

impl CameraOpticalResponse2d {
    pub fn normalized(mut self) -> Self {
        self.intensity = finite_or_zero(self.intensity).clamp(0.0, 8.0);
        self.bloom = finite_or_zero(self.bloom).clamp(0.0, 8.0);
        self.glare = finite_or_zero(self.glare).clamp(0.0, 8.0);
        self.ghosting = finite_or_zero(self.ghosting).clamp(0.0, 8.0);
        self.streaks = finite_or_zero(self.streaks).clamp(0.0, 8.0);
        self.chromatic_smear = finite_or_zero(self.chromatic_smear).clamp(0.0, 8.0);
        self.dirt_response = finite_or_zero(self.dirt_response).clamp(0.0, 8.0);
        self.halation = finite_or_zero(self.halation).clamp(0.0, 8.0);
        self.threshold = finite_or_zero(self.threshold).clamp(0.0, 1.0);
        self
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
