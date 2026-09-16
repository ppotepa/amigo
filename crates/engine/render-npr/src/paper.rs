//! The drawing surface is independent from a brush or a value plan.

use glam::Vec4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprPaperDefinition {
    pub color: Vec4,
    pub tooth_scale: f32,
    pub fibre_strength: f32,
    pub absorption: f32,
}

impl NprPaperDefinition {
    pub fn normalized(self) -> Self {
        Self {
            color: self.color,
            tooth_scale: self.tooth_scale.max(0.01),
            fibre_strength: self.fibre_strength.clamp(0.0, 1.0),
            absorption: self.absorption.clamp(0.0, 1.0),
        }
    }
}

impl Default for NprPaperDefinition {
    fn default() -> Self {
        Self {
            color: Vec4::new(0.92, 0.88, 0.78, 1.0),
            tooth_scale: 1.0,
            fibre_strength: 0.35,
            absorption: 0.75,
        }
    }
}
