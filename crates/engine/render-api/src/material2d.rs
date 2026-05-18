#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material2dOpticalMode {
    Opaque,
    Transmissive,
    Refractive,
    Emissive,
}

impl Default for Material2dOpticalMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material2dOptical {
    pub mode: Material2dOpticalMode,
    pub transmission: f32,
    pub refraction_px: f32,
    pub distortion: f32,
    pub dispersion: f32,
    pub roughness: f32,
    pub edge_boost: f32,
}

impl Default for Material2dOptical {
    fn default() -> Self {
        Self {
            mode: Material2dOpticalMode::Opaque,
            transmission: 0.0,
            refraction_px: 0.0,
            distortion: 0.0,
            dispersion: 0.0,
            roughness: 0.0,
            edge_boost: 0.0,
        }
    }
}

impl Material2dOptical {
    pub fn normalized(mut self) -> Self {
        self.transmission = self.transmission.clamp(0.0, 1.0);
        self.refraction_px = self.refraction_px.max(0.0);
        self.distortion = self.distortion.clamp(0.0, 1.0);
        self.dispersion = self.dispersion.clamp(0.0, 1.0);
        self.roughness = self.roughness.clamp(0.0, 1.0);
        self.edge_boost = self.edge_boost.clamp(0.0, 2.0);
        self
    }

    pub fn is_refractive(self) -> bool {
        self.mode == Material2dOpticalMode::Refractive
            && (self.transmission > 0.0 || self.refraction_px > 0.0 || self.distortion > 0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material2dLighting {
    pub receives_light: bool,
    pub response: f32,
}

impl Default for Material2dLighting {
    fn default() -> Self {
        Self {
            receives_light: false,
            response: 0.0,
        }
    }
}

impl Material2dLighting {
    pub fn normalized(mut self) -> Self {
        self.response = self.response.clamp(0.0, 2.0);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material2dCameraResponse {
    pub highlight: f32,
    pub bloom_source: bool,
    pub rain_glass_affects: bool,
}

impl Default for Material2dCameraResponse {
    fn default() -> Self {
        Self {
            highlight: 0.0,
            bloom_source: false,
            rain_glass_affects: false,
        }
    }
}

impl Material2dCameraResponse {
    pub fn normalized(mut self) -> Self {
        self.highlight = self.highlight.clamp(0.0, 2.0);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Material2d {
    pub optical: Material2dOptical,
    pub lighting: Material2dLighting,
    pub camera_response: Material2dCameraResponse,
}

impl Material2d {
    pub fn normalized(mut self) -> Self {
        self.optical = self.optical.normalized();
        self.lighting = self.lighting.normalized();
        self.camera_response = self.camera_response.normalized();
        self
    }

    pub fn requires_material_mask(self) -> bool {
        self.optical.is_refractive()
            || self.camera_response.highlight > 0.0
            || self.camera_response.bloom_source
    }

    pub fn is_refractive(self) -> bool {
        self.optical.is_refractive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refractive_material_requires_material_mask() {
        let material = Material2d {
            optical: Material2dOptical {
                mode: Material2dOpticalMode::Refractive,
                transmission: 0.6,
                refraction_px: 4.0,
                distortion: 0.2,
                dispersion: 0.1,
                roughness: 0.3,
                edge_boost: 0.4,
            },
            ..Default::default()
        }
        .normalized();

        assert!(material.is_refractive());
        assert!(material.requires_material_mask());
    }

    #[test]
    fn opaque_default_material_does_not_require_mask() {
        let material = Material2d::default().normalized();

        assert!(!material.is_refractive());
        assert!(!material.requires_material_mask());
    }
}
