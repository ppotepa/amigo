use crate::feature::FeatureClass;
use crate::tool::StrokeTool;
use glam::Vec4;

/// Controls whether tonal form is painted as authored colour bands or produced
/// by strokes over the paper. It is a typed rendering decision, not a backend
/// inference based on a preset name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NprToneMode {
    #[default]
    ThreeBand,
    Hatching,
}

/// Declares whether triangle edges describe the intended form or merely sample
/// a smooth one. It is an authored/domain decision, never inferred by WGPU
/// from a mesh name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NprSurfaceMode {
    #[default]
    Polygonal,
    Smooth,
}

/// The authored reading of a model surface.  Unlike [`NprSurfaceMode`], which
/// selects a concrete contour pipeline, this records *why* that pipeline is
/// appropriate.  Extractors resolve it before creating an NPR packet, so a
/// backend never has to infer drawing intent from mesh density or a model name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NprSurfaceIntent {
    /// Preserve deliberately faceted planes and their authored sharp edges.
    HardSurface,
    /// Treat topology as a sampling of a continuous form.  The drawing uses a
    /// smooth proxy and never turns incidental triangle creases into ink.
    Organic,
    /// Respect an explicit [`NprSurfaceMode`] selected by an author.
    #[default]
    Authored,
}

impl NprSurfaceIntent {
    pub const fn resolve_mode(self, authored_mode: NprSurfaceMode) -> NprSurfaceMode {
        match self {
            Self::HardSurface => NprSurfaceMode::Polygonal,
            Self::Organic => NprSurfaceMode::Smooth,
            Self::Authored => authored_mode,
        }
    }

    pub const fn resolve_subdivision_level(self, authored_level: u8) -> u8 {
        match self {
            Self::HardSurface => 0,
            Self::Organic if authored_level == 0 => 1,
            Self::Organic => authored_level,
            Self::Authored => authored_level,
        }
    }

    pub const fn suppresses_topology_creases(self) -> bool {
        matches!(self, Self::Organic)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ComicInk {
    pub tool: StrokeTool,
    pub tone_mode: NprToneMode,
    pub surface_mode: NprSurfaceMode,
    pub light_direction: glam::Vec3,
    pub ink: Vec4,
    pub crease_angle: f32,
    /// Dihedral threshold above which the smooth contour field must not blend
    /// normals across a mesh edge. This is independent from whether that edge
    /// is drawn as an explicit crease.
    pub smooth_crease_angle: f32,
    /// Whether a Smooth drawing may retain topology-derived crease strokes.
    /// Disabled by default: imported organic meshes frequently contain sharp
    /// triangulation artifacts which are not authored drawing intent. The
    /// smooth contour field still honours `smooth_crease_angle` as a normal
    /// discontinuity.
    pub smooth_draw_creases: bool,
    pub paper: Vec4,
    pub shadow: Vec4,
    pub mid: Vec4,
    pub light: Vec4,
    pub outline_width: f32,
    pub crease_width: f32,
    pub boundary_width: f32,
    /// Shorter interior crease chains are omitted before tessellation. This is
    /// a screen-space drawing policy; silhouettes and boundaries are retained.
    pub min_crease_length_pixels: f32,
    /// Short smooth-contour spans tend to be local sampling noise rather than
    /// a readable silhouette. This screen-space gate runs after contour
    /// assembly and before tessellation.
    pub min_smooth_contour_length_pixels: f32,
    /// Pixel tolerance for deterministic contour simplification after
    /// projection. It removes sub-pixel triangulation bends while preserving
    /// the visible path's endpoints and major turns.
    pub smooth_contour_simplification_pixels: f32,
    pub taper: f32,
    pub wobble: f32,
    pub gesture_confidence: f32,
    pub gesture_simplification: f32,
    pub gesture_correction: f32,
    pub gesture_overstroke: f32,
    pub tool_pressure: f32,
    pub tool_hardness: f32,
    pub paper_tooth: f32,
    pub paper_grain: f32,
    pub nib_angle: f32,
    pub nib_aspect: f32,
    pub ink_dryness: f32,
    pub tone_density: f32,
    /// Lowest reliable local form direction accepted for a tonal stroke.
    /// This suppresses arbitrary marks where the inferred tangent is unstable
    /// or the normal field turns too abruptly across sampling triangles.
    pub min_form_line_confidence: f32,
    /// Opt-in interior contours where view-dependent radial curvature crosses
    /// zero on a smooth surface. They are secondary form marks, never a
    /// replacement for the outer silhouette.
    pub suggestive_contours: bool,
    pub suggestive_contour_confidence: f32,
    /// Secondary interior contour appearance, relative to authored crease ink.
    pub suggestive_contour_width_scale: f32,
    pub suggestive_contour_opacity: f32,
    /// Tonal form-line appearance, independent from silhouette/crease ink.
    pub form_line_width_scale: f32,
    pub form_line_opacity: f32,
    pub hatching_angle: f32,
    pub hatching_spacing: f32,
    pub hatching_cross: f32,
}

impl Default for ComicInk {
    fn default() -> Self {
        Self {
            tool: StrokeTool::Fineliner,
            tone_mode: NprToneMode::ThreeBand,
            surface_mode: NprSurfaceMode::Polygonal,
            light_direction: glam::Vec3::new(-0.4, 0.7, 1.0),
            ink: Vec4::new(0.035, 0.025, 0.02, 1.0),
            crease_angle: 0.35,
            smooth_crease_angle: 1.2,
            smooth_draw_creases: false,
            paper: Vec4::new(0.92, 0.88, 0.78, 1.0),
            shadow: Vec4::new(0.18, 0.20, 0.28, 1.0),
            mid: Vec4::new(0.46, 0.54, 0.68, 1.0),
            light: Vec4::new(0.82, 0.84, 0.78, 1.0),
            outline_width: 4.0,
            crease_width: 2.0,
            boundary_width: 3.0,
            min_crease_length_pixels: 4.0,
            min_smooth_contour_length_pixels: 8.0,
            smooth_contour_simplification_pixels: 0.75,
            taper: 0.18,
            wobble: 0.0,
            gesture_confidence: 1.0,
            gesture_simplification: 0.0,
            gesture_correction: 0.0,
            gesture_overstroke: 0.0,
            // A default fineliner is intentionally neutral. Hand-drawn
            // response is opt-in through the Pencil/Brush presets, so the
            // established Comic Ink look and its golden images stay stable.
            tool_pressure: 1.0,
            tool_hardness: 0.85,
            paper_tooth: 0.0,
            paper_grain: 0.0,
            nib_angle: 0.0,
            nib_aspect: 0.0,
            ink_dryness: 0.0,
            tone_density: 0.0,
            min_form_line_confidence: 0.22,
            suggestive_contours: false,
            suggestive_contour_confidence: 0.40,
            suggestive_contour_width_scale: 0.55,
            suggestive_contour_opacity: 0.55,
            form_line_width_scale: 1.0,
            form_line_opacity: 1.0,
            hatching_angle: -25.0,
            hatching_spacing: 9.0,
            hatching_cross: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_intent_resolves_without_mesh_name_heuristics() {
        assert_eq!(
            NprSurfaceIntent::HardSurface.resolve_mode(NprSurfaceMode::Smooth),
            NprSurfaceMode::Polygonal
        );
        assert_eq!(
            NprSurfaceIntent::Organic.resolve_mode(NprSurfaceMode::Polygonal),
            NprSurfaceMode::Smooth
        );
        assert_eq!(
            NprSurfaceIntent::Authored.resolve_mode(NprSurfaceMode::Smooth),
            NprSurfaceMode::Smooth
        );
        assert_eq!(NprSurfaceIntent::HardSurface.resolve_subdivision_level(2), 0);
        assert_eq!(NprSurfaceIntent::Organic.resolve_subdivision_level(0), 1);
        assert!(!NprSurfaceIntent::HardSurface.suppresses_topology_creases());
        assert!(NprSurfaceIntent::Organic.suppresses_topology_creases());
    }
}
impl ComicInk {
    pub fn width(self, class: FeatureClass) -> f32 {
        match class {
            FeatureClass::Boundary => self.boundary_width,
            FeatureClass::Silhouette => self.outline_width,
            FeatureClass::Crease => self.crease_width,
        }
    }
}
