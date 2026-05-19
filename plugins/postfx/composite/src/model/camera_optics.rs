#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraOptics2d {
    pub focal_length_mm: f32,
    pub aberration_px: f32,
    pub distortion: f32,
    pub vignette: f32,
    pub edge_softness_px: f32,
    pub glare_strength: f32,
    pub lens_bloom: f32,
    pub flare_ghosts: f32,
    pub anamorphic_squeeze: f32,
    pub coma: f32,
    pub dirt: f32,
    pub halation_bias: f32,
    pub opacity: f32,
}

impl Default for CameraOptics2d {
    fn default() -> Self {
        Self {
            focal_length_mm: 35.0,
            aberration_px: 0.0,
            distortion: 0.0,
            vignette: 0.0,
            edge_softness_px: 0.0,
            glare_strength: 0.0,
            lens_bloom: 0.0,
            flare_ghosts: 0.0,
            anamorphic_squeeze: 1.0,
            coma: 0.0,
            dirt: 0.0,
            halation_bias: 0.0,
            opacity: 1.0,
        }
    }
}

impl CameraOptics2d {
    pub fn normalized(mut self) -> Self {
        self.focal_length_mm = self.focal_length_mm.clamp(8.0, 400.0);
        self.aberration_px = self.aberration_px.clamp(0.0, 8.0);
        self.distortion = self.distortion.clamp(-0.5, 0.5);
        self.vignette = self.vignette.clamp(0.0, 2.0);
        self.edge_softness_px = self.edge_softness_px.clamp(0.0, 16.0);
        self.glare_strength = self.glare_strength.clamp(0.0, 2.0);
        self.lens_bloom = self.lens_bloom.clamp(0.0, 2.0);
        self.flare_ghosts = self.flare_ghosts.clamp(0.0, 2.0);
        self.anamorphic_squeeze = self.anamorphic_squeeze.clamp(1.0, 3.0);
        self.coma = self.coma.clamp(0.0, 2.0);
        self.dirt = self.dirt.clamp(0.0, 1.0);
        self.halation_bias = self.halation_bias.clamp(0.0, 1.0);
        self.opacity = self.opacity.clamp(0.0, 1.0);
        self
    }

    pub fn is_active(&self) -> bool {
        self.opacity > 0.0
            && (self.aberration_px > 0.0
                || self.distortion.abs() > 0.0
                || self.vignette > 0.0
                || self.edge_softness_px > 0.0
                || self.glare_strength > 0.0
                || self.lens_bloom > 0.0
                || self.flare_ghosts > 0.0
                || (self.anamorphic_squeeze - 1.0).abs() > f32::EPSILON
                || self.coma > 0.0
                || self.dirt > 0.0
                || self.halation_bias > 0.0)
    }
}
