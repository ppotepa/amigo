use amigo_camera::CameraOpticalResponse2d;

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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Material2d {
    pub optical: Material2dOptical,
    pub lighting: Material2dLighting,
    pub camera_response: CameraOpticalResponse2d,
}

impl Material2d {
    pub fn normalized(mut self) -> Self {
        self.optical = self.optical.normalized();
        self.lighting = self.lighting.normalized();
        self.camera_response = self.camera_response.normalized();
        self
    }

    pub fn requires_material_mask(self) -> bool {
        let response = self.camera_response.normalized();
        self.optical.is_refractive()
            || (response.enabled
                && (response.intensity > 0.0
                    || response.glare > 0.0
                    || response.bloom > 0.0
                    || response.dirt_response > 0.0))
    }

    pub fn is_refractive(self) -> bool {
        self.optical.is_refractive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialCoverageKind2d {
    Glyphs,
    TextureAlpha,
    VectorCoverage,
    LayeredImageAlpha,
    ParticleCoverage,
    Unsupported,
}

impl MaterialCoverageKind2d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Glyphs => "glyphs",
            Self::TextureAlpha => "texture_alpha",
            Self::VectorCoverage => "vector_coverage",
            Self::LayeredImageAlpha => "layered_image_alpha",
            Self::ParticleCoverage => "particle_coverage",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialCandidateStatus2d {
    Active,
    Skipped,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialCandidateDecision2d {
    pub owner: String,
    pub component_kind: String,
    pub render_layer: String,
    pub coverage_kind: MaterialCoverageKind2d,
    pub status: MaterialCandidateStatus2d,
    pub reason: String,
}

impl MaterialCandidateDecision2d {
    pub fn active(
        owner: impl Into<String>,
        component_kind: impl Into<String>,
        render_layer: impl Into<String>,
        coverage_kind: MaterialCoverageKind2d,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            component_kind: component_kind.into(),
            render_layer: render_layer.into(),
            coverage_kind,
            status: MaterialCandidateStatus2d::Active,
            reason: reason.into(),
        }
    }

    pub fn skipped(
        owner: impl Into<String>,
        component_kind: impl Into<String>,
        render_layer: impl Into<String>,
        coverage_kind: MaterialCoverageKind2d,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            component_kind: component_kind.into(),
            render_layer: render_layer.into(),
            coverage_kind,
            status: MaterialCandidateStatus2d::Skipped,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialCandidate2dCommon {
    pub owner: String,
    pub component_kind: String,
    pub render_layer: String,
    pub z_index: f32,
    pub layer_opacity: f32,
    pub visible: bool,
    pub material: Material2d,
    pub coverage_kind: MaterialCoverageKind2d,
}

impl MaterialCandidate2dCommon {
    pub fn is_refractive(&self) -> bool {
        self.visible && self.layer_opacity > 0.001 && self.material.is_refractive()
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

    #[test]
    fn material_candidate_common_reports_coverage_kind() {
        let candidate = MaterialCandidate2dCommon {
            owner: "title".to_owned(),
            component_kind: "Text2D".to_owned(),
            render_layer: "title.depth2d".to_owned(),
            z_index: 10.0,
            layer_opacity: 0.72,
            visible: true,
            material: Material2d::default(),
            coverage_kind: MaterialCoverageKind2d::Glyphs,
        };

        assert_eq!(candidate.coverage_kind.as_str(), "glyphs");
    }
}
