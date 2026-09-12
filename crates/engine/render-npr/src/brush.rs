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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum GeometryTarget {
    All,
    Objects(Vec<String>),
    SurfaceFeatures(Vec<String>),
}
impl Default for GeometryTarget {
    fn default() -> Self {
        Self::All
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
    Multiply(Vec<CoverageMask>),
}
impl Default for CoverageMask {
    fn default() -> Self {
        Self::None
    }
}
impl CoverageMask {
    /// Deterministic, view-independent mask evaluation. The extractor supplies
    /// normalized tone/height/normal and an object-local noise coordinate.
    pub fn evaluate(&self, tone: f32, height: f32, normal: [f32; 3], noise: f32) -> f32 {
        let clamp = |v: f32| v.clamp(0.0, 1.0);
        match self {
            Self::None => 1.0,
            Self::ToneRange { min, max, invert } => {
                let v = if (*min..=*max).contains(&tone) {
                    1.0
                } else {
                    0.0
                };
                if *invert { 1.0 - v } else { v }
            }
            Self::Height { min, max, invert } => {
                let v = if (*min..=*max).contains(&height) {
                    1.0
                } else {
                    0.0
                };
                if *invert { 1.0 - v } else { v }
            }
            Self::NormalDirection {
                direction,
                threshold,
                invert,
            } => {
                let dot = normal
                    .iter()
                    .zip(direction)
                    .map(|(a, b)| a * b)
                    .sum::<f32>();
                let v = clamp((dot - threshold) / (1.0 - threshold).max(0.0001));
                if *invert { 1.0 - v } else { v }
            }
            Self::Noise { amount, invert, .. } => {
                let v = clamp(noise + amount * (noise * 17.0).sin());
                if *invert { 1.0 - v } else { v }
            }
            Self::Multiply(masks) => masks
                .iter()
                .map(|mask| mask.evaluate(tone, height, normal, noise))
                .product(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
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
            } if direction.iter().any(|v| !v.is_finite()) || !threshold.is_finite() => {
                Err("normal mask is invalid".into())
            }
            Self::Noise { amount, .. } if !amount.is_finite() || !(0. ..=1.).contains(amount) => {
                Err("noise mask amount is invalid".into())
            }
            Self::Multiply(masks) => masks.iter().try_for_each(Self::validate),
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
        self.brushes
            .entry(brush.id.clone())
            .or_default()
            .push(brush);
        Ok(())
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
    #[test]
    fn masks_are_deterministic_and_composable() {
        let mask = CoverageMask::Multiply(vec![
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
        ]);
        assert_eq!(
            mask.evaluate(0.5, 0.0, [0., 0., 1.], 0.5),
            mask.evaluate(0.5, 0.0, [0., 0., 1.], 0.5)
        );
        assert_eq!(mask.evaluate(0.1, 0.0, [0., 0., 1.], 0.5), 0.0);
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
        assert_eq!(tone.evaluate(0.1, 0.0, [0.0, 0.0, 1.0], 0.0), 0.0);
        assert_eq!(tone.evaluate(0.5, 0.0, [0.0, 0.0, 1.0], 0.0), 1.0);
        let height = CoverageMask::Height {
            min: 0.2,
            max: 0.8,
            invert: true,
        };
        assert_eq!(height.evaluate(0.5, 0.0, [0.0, 0.0, 1.0], 0.0), 1.0);
        assert_eq!(height.evaluate(0.5, 0.5, [0.0, 0.0, 1.0], 0.0), 0.0);
        let normal = CoverageMask::NormalDirection {
            direction: [0.0, 0.0, 1.0],
            threshold: 0.5,
            invert: false,
        };
        assert_eq!(normal.evaluate(0.5, 0.0, [0.0, 0.0, 1.0], 0.0), 1.0);
        assert_eq!(normal.evaluate(0.5, 0.0, [0.0, 1.0, 0.0], 0.0), 0.0);
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
