use crate::feature::FeatureClass;
use crate::tool::StrokeTool;
use glam::Vec4;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ComicInk {
    pub tool: StrokeTool,
    pub light_direction: glam::Vec3,
    pub ink: Vec4,
    pub crease_angle: f32,
    pub paper: Vec4,
    pub shadow: Vec4,
    pub mid: Vec4,
    pub light: Vec4,
    pub outline_width: f32,
    pub crease_width: f32,
    pub boundary_width: f32,
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
    pub hatching_angle: f32,
    pub hatching_spacing: f32,
    pub hatching_cross: f32,
}

impl Default for ComicInk {
    fn default() -> Self {
        Self {
            tool: StrokeTool::Fineliner,
            light_direction: glam::Vec3::new(-0.4, 0.7, 1.0),
            ink: Vec4::new(0.035, 0.025, 0.02, 1.0),
            crease_angle: 0.35,
            paper: Vec4::new(0.92, 0.88, 0.78, 1.0),
            shadow: Vec4::new(0.18, 0.20, 0.28, 1.0),
            mid: Vec4::new(0.46, 0.54, 0.68, 1.0),
            light: Vec4::new(0.82, 0.84, 0.78, 1.0),
            outline_width: 4.0,
            crease_width: 2.0,
            boundary_width: 3.0,
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
            hatching_angle: -25.0,
            hatching_spacing: 9.0,
            hatching_cross: 0.0,
        }
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
