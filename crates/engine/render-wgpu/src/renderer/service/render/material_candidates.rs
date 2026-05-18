use amigo_render_api::Material2d;

#[derive(Debug, Clone)]
pub(super) enum MaterialCoverageSource2d {
    Glyphs {
        entity_name: String,
        render_layer: String,
    },
    TextureAlpha {
        entity_name: String,
        render_layer: String,
    },
    VectorCoverage {
        entity_name: String,
        render_layer: String,
    },
}

#[derive(Debug, Clone)]
pub(super) struct MaterialCandidate2d {
    pub(super) entity_name: String,
    pub(super) component_kind: &'static str,
    pub(super) render_layer: String,
    pub(super) z_index: f32,
    pub(super) material: Material2d,
    pub(super) coverage_source: MaterialCoverageSource2d,
    pub(super) layer_opacity: f32,
    pub(super) visible: bool,
}

impl MaterialCandidate2d {
    pub(super) fn is_refractive(&self) -> bool {
        self.visible
            && self.layer_opacity > 0.001
            && self.material.is_refractive()
    }

    pub(super) fn coverage_label(&self) -> &'static str {
        match self.coverage_source {
            MaterialCoverageSource2d::Glyphs { .. } => "glyphs",
            MaterialCoverageSource2d::TextureAlpha { .. } => "texture_alpha",
            MaterialCoverageSource2d::VectorCoverage { .. } => "vector_coverage",
        }
    }
}
