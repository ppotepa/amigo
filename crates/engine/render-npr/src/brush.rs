//! Authoring contracts for procedural brushes and stable coverage masks.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrushMedium {
    Ink,
    Graphite,
    Hatching,
    FlatFill,
    WatercolourWash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrushApplication {
    Stroke,
    Surface,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrushDefinition {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub medium: BrushMedium,
    pub applications: Vec<BrushApplication>,
    /// A reusable appearance never chooses a geometric source.
    pub tool: Option<crate::StrokeTool>,
    pub paint: Option<crate::NprPaintMedium>,
    pub width: f32,
    pub taper: f32,
    pub softness: f32,
    pub spacing: f32,
    pub pressure_profile: f32,
    pub irregularity: f32,
    pub correction: f32,
    pub dryness: f32,
    pub seed: u64,
}

impl Default for BrushDefinition {
    fn default() -> Self {
        Self {
            id: "builtin-ink".into(),
            name: "Ink liner".into(),
            version: 1,
            medium: BrushMedium::Ink,
            applications: vec![BrushApplication::Stroke],
            tool: None,
            paint: None,
            width: 1.0,
            taper: 0.2,
            softness: 0.1,
            spacing: 0.25,
            pressure_profile: 0.5,
            irregularity: 0.1,
            correction: 0.2,
            dryness: 0.3,
            seed: 0x4e5052,
        }
    }
}

impl BrushDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || self.version == 0 || self.name.trim().is_empty() {
            return Err("brush id, name and version are required".into());
        }
        if self.applications.is_empty() {
            return Err("brush must support at least one application".into());
        }
        if self.paint.is_some_and(|paint| {
            !paint.wash.is_finite()
                || !(0.0..=2.0).contains(&paint.wash)
                || !paint.granulation.is_finite()
                || !(0.0..=1.0).contains(&paint.granulation)
        }) {
            return Err("invalid appearance paint response".into());
        }
        for (label, value, range) in [
            ("width", self.width, (0.001, 100.)),
            ("taper", self.taper, (0., 1.)),
            ("softness", self.softness, (0., 1.)),
            ("spacing", self.spacing, (0.01, 4.)),
            ("pressure_profile", self.pressure_profile, (0., 1.)),
            ("irregularity", self.irregularity, (0., 1.)),
            ("correction", self.correction, (0., 1.)),
            ("dryness", self.dryness, (0., 1.)),
        ] {
            if !value.is_finite() || value < range.0 || value > range.1 {
                return Err(format!("invalid brush {label}"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BrushReference {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrushInstance {
    pub brush: BrushReference,
    pub width: Option<f32>,
    pub taper: Option<f32>,
    pub softness: Option<f32>,
    pub irregularity: Option<f32>,
    pub dryness: Option<f32>,
    pub seed: Option<u64>,
    pub pressure_profile: Option<f32>,
    pub correction: Option<f32>,
    pub spacing: Option<f32>,
}

impl BrushMedium {
    pub fn tool(self) -> crate::StrokeTool {
        match self {
            Self::Ink => crate::StrokeTool::Fineliner,
            Self::Graphite | Self::Hatching => crate::StrokeTool::Pencil,
            Self::FlatFill | Self::WatercolourWash => crate::StrokeTool::Brush,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum GeometryTarget {
    All,
    Objects { objects: Vec<String> },
    SurfaceFeatures { features: Vec<String> },
}
impl Default for GeometryTarget {
    fn default() -> Self {
        Self::All
    }
}

impl GeometryTarget {
    pub fn includes(&self, object_id: &str, source: crate::NprGeometrySource) -> bool {
        match self {
            Self::All => true,
            Self::Objects { objects } => objects.iter().any(|id| id == object_id),
            Self::SurfaceFeatures { features } => {
                features.iter().any(|feature| feature == source.key())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum CoverageMask {
    None,
    ToneRange {
        min: f32,
        max: f32,
        invert: bool,
    },
    Height {
        min: f32,
        max: f32,
        invert: bool,
    },
    NormalDirection {
        direction: [f32; 3],
        threshold: f32,
        invert: bool,
    },
    Noise {
        amount: f32,
        seed: u64,
        invert: bool,
    },
    Multiply {
        masks: Vec<CoverageMask>,
    },
}
impl Default for CoverageMask {
    fn default() -> Self {
        Self::None
    }
}
impl CoverageMask {
    /// Conservative exclusion test: an unsampled narrow band must not be
    /// diagnosed as invisible when fragments inside it can still contribute.
    pub fn triangle_may_cover(&self, samples: [Option<crate::NprCoverageSample>; 3]) -> bool {
        if !self.requires_surface() {
            return true;
        }
        let [Some(a), Some(b), Some(c)] = samples else {
            return false;
        };
        match self {
            Self::ToneRange { min, max, invert } | Self::Height { min, max, invert } => {
                let values = [a, b, c].map(|s| {
                    if matches!(self, Self::Height { .. }) {
                        s.height
                    } else {
                        s.tone
                    }
                });
                let low = values.into_iter().fold(f32::INFINITY, f32::min);
                let high = values.into_iter().fold(f32::NEG_INFINITY, f32::max);
                if *invert {
                    low < *min || high > *max
                } else {
                    low <= *max && high >= *min
                }
            }
            Self::NormalDirection { .. } => {
                a.normal != b.normal || a.normal != c.normal || self.evaluate(Some(a)) > 0.0
            }
            Self::Noise { amount, invert, .. } => !(*invert && *amount == 0.0),
            Self::Multiply { masks } => masks.iter().all(|m| m.triangle_may_cover(samples)),
            Self::None => true,
        }
    }
    /// Screen-space quadrature used for diagnostics, not a replacement for the
    /// per-fragment evaluator. Returns an explicitly approximate coverage.
    pub fn triangle_estimate(
        &self,
        samples: [Option<crate::NprCoverageSample>; 3],
        depths: [f32; 3],
    ) -> f32 {
        if !self.requires_surface() {
            return 1.0;
        }
        let mut sum = 0.0;
        for row in 0..8 {
            for column in 0..8 - row {
                for offset in [1.0 / 3.0, 2.0 / 3.0] {
                    if offset > 0.5 && row + column == 7 {
                        continue;
                    }
                    let b = (row as f32 + offset) / 8.0;
                    let c = (column as f32 + offset) / 8.0;
                    sum += self.evaluate(crate::NprCoverageSample::interpolate(
                        samples,
                        depths,
                        [1.0 - b - c, b, c],
                    ));
                }
            }
        }
        sum / 64.0
    }
    pub fn term_count(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Multiply { masks } => masks.iter().map(Self::term_count).sum(),
            _ => 1,
        }
    }
    pub fn requires_surface(&self) -> bool {
        match self {
            Self::None => false,
            Self::Multiply { masks } => masks.iter().any(Self::requires_surface),
            _ => true,
        }
    }
    /// All consumers use the extractor's object-local inputs. Missing inputs
    /// clip coverage and are reported by layer validation, never fabricated.
    pub fn evaluate(&self, sample: Option<crate::NprCoverageSample>) -> f32 {
        if !self.requires_surface() {
            return 1.0;
        }
        let Some(sample) = sample else {
            return 0.0;
        };
        let invert_value = |value: f32, invert: bool| if invert { 1.0 - value } else { value };
        match self {
            Self::None => 1.0,
            Self::ToneRange { min, max, invert } => invert_value(
                if (*min..=*max).contains(&sample.tone) {
                    1.0
                } else {
                    0.0
                },
                *invert,
            ),
            Self::Height { min, max, invert } => invert_value(
                if (*min..=*max).contains(&sample.height) {
                    1.0
                } else {
                    0.0
                },
                *invert,
            ),
            Self::NormalDirection {
                direction,
                threshold,
                invert,
            } => {
                let direction = glam::Vec3::from_array(*direction).normalize_or_zero();
                let dot = sample.normal.normalize_or_zero().dot(direction);
                let value = if *threshold >= 1.0 {
                    if dot >= 1.0 - 1e-6 { 1.0 } else { 0.0 }
                } else {
                    ((dot - threshold) / (1.0 - threshold)).clamp(0.0, 1.0)
                };
                invert_value(value, *invert)
            }
            Self::Noise {
                amount,
                seed,
                invert,
            } => invert_value(
                1.0 - amount + amount * crate::coverage::coverage_noise(sample.position, *seed),
                *invert,
            ),
            Self::Multiply { masks } => masks
                .iter()
                .map(|mask| mask.evaluate(Some(sample)))
                .product(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.term_count() > 128 {
            return Err("coverage mask exceeds 128 terms".into());
        }
        match self {
            Self::ToneRange { min, max, .. } | Self::Height { min, max, .. }
                if !min.is_finite() || !max.is_finite() || min > max =>
            {
                Err("mask range is invalid".into())
            }
            Self::NormalDirection {
                direction,
                threshold,
                ..
            } if direction.iter().any(|v| !v.is_finite())
                || glam::Vec3::from_array(*direction).length_squared() <= 1e-12
                || !glam::Vec3::from_array(*direction)
                    .length_squared()
                    .is_finite()
                || !threshold.is_finite()
                || !(-1.0..=1.0).contains(threshold) =>
            {
                Err("normal mask is invalid".into())
            }
            Self::Noise { amount, .. } if !amount.is_finite() || !(0. ..=1.).contains(amount) => {
                Err("noise mask amount is invalid".into())
            }
            Self::Multiply { masks } => masks.iter().try_for_each(Self::validate),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrushLibrary {
    pub brushes: BTreeMap<String, Vec<BrushDefinition>>,
}
impl BrushLibrary {
    /// The immutable first-stage Drawing Studio library. Documents may add a
    /// newer version, but references always name this concrete version.
    pub fn drawing_studio() -> Self {
        let mut library = Self::default();
        for (id, name, medium, applications, width, dryness) in [
            (
                "ink-liner",
                "Ink liner",
                BrushMedium::Ink,
                vec![BrushApplication::Stroke],
                1.2,
                0.25,
            ),
            (
                "graphite-study",
                "Graphite study",
                BrushMedium::Graphite,
                vec![BrushApplication::Stroke],
                1.8,
                0.8,
            ),
            (
                "surface-hatch",
                "Surface hatching",
                BrushMedium::Hatching,
                vec![BrushApplication::Stroke, BrushApplication::Surface],
                1.0,
                0.6,
            ),
            (
                "flat-fill",
                "Flat fill",
                BrushMedium::FlatFill,
                vec![BrushApplication::Surface],
                1.0,
                0.0,
            ),
            (
                "watercolour-wash",
                "Watercolour wash",
                BrushMedium::WatercolourWash,
                vec![BrushApplication::Surface],
                1.0,
                0.9,
            ),
        ] {
            library
                .add_version(BrushDefinition {
                    id: id.into(),
                    name: name.into(),
                    medium,
                    applications,
                    width,
                    dryness,
                    ..BrushDefinition::default()
                })
                .expect("built-in Drawing Studio brushes are valid");
        }
        library
    }

    pub fn add_version(&mut self, brush: BrushDefinition) -> Result<(), String> {
        brush.validate()?;
        if self
            .brushes
            .get(&brush.id)
            .is_some_and(|versions| versions.iter().any(|old| old.version == brush.version))
        {
            return Err(format!(
                "brush version already exists: {}@{}",
                brush.id, brush.version
            ));
        }
        self.brushes
            .entry(brush.id.clone())
            .or_default()
            .push(brush);
        Ok(())
    }
    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        let mut merged = self.clone();
        for (id, versions) in &other.brushes {
            for brush in versions {
                if &brush.id != id {
                    return Err("brush library key does not match its definition".into());
                }
                let reference = BrushReference {
                    id: id.clone(),
                    version: brush.version,
                };
                match merged.resolve(&reference) {
                    Ok(existing) if existing != brush => {
                        return Err(format!(
                            "conflicting immutable brush: {id}@{}",
                            brush.version
                        ));
                    }
                    Ok(_) => (),
                    Err(_) => merged.add_version(brush.clone())?,
                }
            }
        }
        *self = merged;
        Ok(())
    }
    pub fn referenced_by(&self, layers: &crate::NprStyleLayers) -> Result<Self, String> {
        let mut result = Self::default();
        for instance in layers
            .layers
            .iter()
            .filter_map(|layer| layer.brush.as_ref())
        {
            if result.resolve(&instance.brush).is_err() {
                result.add_version(self.resolve(&instance.brush)?.clone())?;
            }
        }
        Ok(result)
    }
    pub fn resolve(&self, reference: &BrushReference) -> Result<&BrushDefinition, String> {
        self.brushes
            .get(&reference.id)
            .and_then(|v| v.iter().find(|b| b.version == reference.version))
            .ok_or_else(|| {
                format!(
                    "missing brush version {}@{}",
                    reference.id, reference.version
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(tone: f32, height: f32, normal: [f32; 3]) -> Option<crate::NprCoverageSample> {
        Some(crate::NprCoverageSample {
            tone,
            height,
            normal: glam::Vec3::from_array(normal),
            position: glam::Vec3::ZERO,
        })
    }
    #[test]
    fn masks_are_deterministic_and_composable() {
        let mask = CoverageMask::Multiply {
            masks: vec![
                CoverageMask::ToneRange {
                    min: 0.2,
                    max: 0.8,
                    invert: false,
                },
                CoverageMask::Noise {
                    amount: 0.0,
                    seed: 7,
                    invert: false,
                },
            ],
        };
        assert_eq!(
            mask.evaluate(sample(0.5, 0.0, [0., 0., 1.])),
            mask.evaluate(sample(0.5, 0.0, [0., 0., 1.]))
        );
        assert_eq!(mask.evaluate(sample(0.1, 0.0, [0., 0., 1.])), 0.0);
        assert!(
            CoverageMask::Noise {
                amount: 2.0,
                seed: 0,
                invert: false
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn masks_cover_tone_height_normal_and_inversion_contracts() {
        let tone = CoverageMask::ToneRange {
            min: 0.25,
            max: 0.75,
            invert: false,
        };
        assert_eq!(tone.evaluate(sample(0.1, 0.0, [0.0, 0.0, 1.0])), 0.0);
        assert_eq!(tone.evaluate(sample(0.5, 0.0, [0.0, 0.0, 1.0])), 1.0);
        let height = CoverageMask::Height {
            min: 0.2,
            max: 0.8,
            invert: true,
        };
        assert_eq!(height.evaluate(sample(0.5, 0.0, [0.0, 0.0, 1.0])), 1.0);
        assert_eq!(height.evaluate(sample(0.5, 0.5, [0.0, 0.0, 1.0])), 0.0);
        let normal = CoverageMask::NormalDirection {
            direction: [0.0, 0.0, 1.0],
            threshold: 0.5,
            invert: false,
        };
        assert_eq!(normal.evaluate(sample(0.5, 0.0, [0.0, 0.0, 1.0])), 1.0);
        assert_eq!(normal.evaluate(sample(0.5, 0.0, [0.0, 1.0, 0.0])), 0.0);
    }

    #[test]
    fn brush_library_requires_pinned_versions() {
        let mut library = BrushLibrary::default();
        library
            .add_version(BrushDefinition {
                id: "ink".into(),
                version: 2,
                ..BrushDefinition::default()
            })
            .unwrap();
        assert!(
            library
                .resolve(&BrushReference {
                    id: "ink".into(),
                    version: 1
                })
                .is_err()
        );
        assert!(
            library
                .resolve(&BrushReference {
                    id: "ink".into(),
                    version: 2
                })
                .is_ok()
        );
    }
}
