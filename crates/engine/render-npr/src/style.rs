use crate::feature::FeatureClass;
use glam::Vec4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComicInk {
    pub paper: Vec4,
    pub shadow: Vec4,
    pub mid: Vec4,
    pub light: Vec4,
    pub outline_width: f32,
    pub crease_width: f32,
    pub boundary_width: f32,
    pub taper: f32,
    pub wobble: f32,
}

impl Default for ComicInk {
    fn default() -> Self {
        Self {
            paper: Vec4::new(0.92, 0.88, 0.78, 1.0),
            shadow: Vec4::new(0.18, 0.20, 0.28, 1.0),
            mid: Vec4::new(0.46, 0.54, 0.68, 1.0),
            light: Vec4::new(0.82, 0.84, 0.78, 1.0),
            outline_width: 4.0,
            crease_width: 2.0,
            boundary_width: 3.0,
            taper: 0.18,
            wobble: 0.0,
        }
    }
}
impl ComicInk {
    pub fn width(self, class: FeatureClass) -> f32 {
        match class {
            FeatureClass::Boundary | FeatureClass::Silhouette => self.boundary_width,
            FeatureClass::Crease => self.crease_width,
        }
    }
}
