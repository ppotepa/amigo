//! Brush selection is separate from medium deposition.

use crate::{FeatureClass, GraphiteMedium, NprMediumDefinition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NprBrushRole {
    Silhouette,
    Crease,
    Hatching,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprBrushDefinition {
    pub role: NprBrushRole,
    pub medium: NprMediumDefinition,
    pub width_scale: f32,
    pub taper: f32,
}

pub trait NprBrushLibrary: Send + Sync {
    fn brush_for(&self, class: FeatureClass) -> NprBrushDefinition;
}

/// A small but real multi-brush pencil palette: softer graphite for selected
/// contours, tighter H/HB-like graphite for construction and hatch marks.
pub struct PencilBrushLibrary;

impl NprBrushLibrary for PencilBrushLibrary {
    fn brush_for(&self, class: FeatureClass) -> NprBrushDefinition {
        match class {
            FeatureClass::Boundary | FeatureClass::Silhouette => NprBrushDefinition {
                role: NprBrushRole::Silhouette,
                medium: NprMediumDefinition::Graphite(GraphiteMedium {
                    hardness: 0.36,
                    deposit_gain: 0.86,
                    tip_radius: 0.72,
                    filament_count: 5,
                    ..Default::default()
                }),
                width_scale: 1.18,
                taper: 0.30,
            },
            FeatureClass::Crease => NprBrushDefinition {
                role: NprBrushRole::Crease,
                medium: NprMediumDefinition::Graphite(GraphiteMedium {
                    hardness: 0.72,
                    deposit_gain: 0.54,
                    tip_radius: 0.35,
                    filament_count: 2,
                    ..Default::default()
                }),
                width_scale: 0.82,
                taper: 0.20,
            },
            FeatureClass::Hatching => NprBrushDefinition {
                role: NprBrushRole::Hatching,
                medium: NprMediumDefinition::Graphite(GraphiteMedium {
                    hardness: 0.58,
                    deposit_gain: 0.48,
                    tip_radius: 0.28,
                    filament_count: 2,
                    ..Default::default()
                }),
                width_scale: 0.62,
                taper: 0.34,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pencil_contours_and_hatches_do_not_share_one_brush() {
        let brushes = PencilBrushLibrary;
        let contour = brushes.brush_for(FeatureClass::Silhouette);
        let hatch = brushes.brush_for(FeatureClass::Hatching);
        assert!(contour.width_scale > hatch.width_scale);
        let NprMediumDefinition::Graphite(contour_medium) = contour.medium else { panic!("expected graphite") };
        let NprMediumDefinition::Graphite(hatch_medium) = hatch.medium else { panic!("expected graphite") };
        assert!(contour_medium.deposit_gain > hatch_medium.deposit_gain);
    }
}
