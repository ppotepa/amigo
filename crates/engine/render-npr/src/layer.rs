//! Typed compositing intent for an NPR drawing.
//!
//! These types describe authorial layers before a render backend decides how a
//! pass is executed. They deliberately do not contain WGPU resources, shader
//! names, or inferred material policy.

use crate::{
    BrushApplication, BrushInstance, BrushLibrary, ComicInk, CoverageMask, FeatureClass,
    GeometryTarget, NprRenderPacket, StrokeRole, StrokeTool, TessellatedStroke,
};
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The model-derived geometry a compositing layer consumes.  This names no
/// rendering medium: identical sources can feed several independent layers.
pub enum NprGeometrySource {
    Paper,
    Wash,
    FlatFill,
    ShadowHatch,
    Silhouette,
    Creases,
    FormLines,
    Construction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NprBlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "color")]
pub enum NprLayerColorSource {
    /// The layer uses the named palette color resolved from its parent style.
    StylePalette,
    /// Use the explicitly declared material base colour. Texture-backed
    /// material sampling requires a future asset adapter contribution.
    ModelBaseColor,
    Constant(Vec4),
}

/// Authored response of a translucent paint medium. It is intentionally
/// compact: the domain applies it to paint coverage before the backend sees
/// geometry, while layer opacity and blend remain compositor concerns.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprPaintMedium {
    pub wash: f32,
    pub granulation: f32,
}

/// Per-layer selection of an already extracted shadow-hatch source. Spacing
/// controls deterministic thinning, so two layers can share the exact source
/// paths while producing different tonal density.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprHatchSettings {
    pub density: f32,
    pub spacing: f32,
}

impl Default for NprHatchSettings {
    fn default() -> Self {
        Self {
            density: 1.0,
            spacing: 1.0,
        }
    }
}

impl Default for NprPaintMedium {
    fn default() -> Self {
        Self {
            wash: 1.0,
            granulation: 0.0,
        }
    }
}

impl Default for NprLayerColorSource {
    fn default() -> Self {
        Self::StylePalette
    }
}

/// One visible authoring layer. `id` is a stable authored identity; `label`
/// may be renamed by a user without invalidating an override or macro target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprStyleLayer {
    pub id: String,
    pub label: String,
    pub source: NprGeometrySource,
    #[serde(default = "enabled")]
    pub enabled: bool,
    #[serde(default = "one")]
    pub opacity: f32,
    #[serde(default)]
    pub blend: NprBlendMode,
    #[serde(default)]
    pub color_source: NprLayerColorSource,
    /// A layer may select a concrete drawing tool without changing the
    /// containing style's default tool. Paint-only layers leave this `None`.
    #[serde(default)]
    pub tool: Option<StrokeTool>,
    /// Paint-only response. Stroke layers leave this unset rather than
    /// receiving an accidental paint treatment.
    #[serde(default)]
    pub paint: Option<NprPaintMedium>,
    #[serde(default)]
    pub hatch: Option<NprHatchSettings>,
    #[serde(default)]
    pub brush: Option<BrushInstance>,
    #[serde(default)]
    pub target: GeometryTarget,
    #[serde(default)]
    pub mask: CoverageMask,
}

/// Editor-facing evidence for one compositing layer. It is intentionally
/// renderer-neutral: the backend may draw the contribution differently, but
/// cannot invent a reason for a missing authored layer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NprLayerDiagnostics {
    pub source_geometry: usize,
    pub generated_marks: usize,
    pub generated_triangles: usize,
    pub mask_coverage: f32,
    pub extraction_micros: u64,
    pub tessellation_micros: u64,
    pub no_effect_reason: Option<NprLayerNoEffectReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NprLayerNoEffectReason {
    Disabled,
    NoSourceGeometry,
    MaskExcludesAll,
    ZeroOpacity,
    HatchSelectionExcludesAll,
}

/// Sparse object-level changes to an inherited layer stack. Properties remain
/// attached to stable layer IDs, so a later scene-level palette or opacity edit
/// continues to reach objects that did not override that specific property.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprStyleLayerOverrides {
    pub layers: BTreeMap<String, NprStyleLayerOverride>,
    /// Reordering is structural rather than a scalar property. Once authored,
    /// it explicitly names the complete inherited set of layer identities.
    pub order: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprStyleLayerOverride {
    pub enabled: Option<bool>,
    pub opacity: Option<f32>,
    pub blend: Option<NprBlendMode>,
    pub color_source: Option<NprLayerColorSource>,
    /// `Some(None)` explicitly clears an inherited tool; `None` continues to
    /// inherit it. This distinction matters when a look turns a media layer
    /// into paint-only output.
    pub tool: Option<Option<StrokeTool>>,
    pub paint: Option<Option<NprPaintMedium>>,
}

impl NprStyleLayerOverrides {
    pub fn is_empty(&self) -> bool {
        self.layers
            .values()
            .all(|layer| layer == &NprStyleLayerOverride::default())
            && self.order.is_none()
    }

    pub fn resolve(&self, inherited: &NprStyleLayers) -> Result<NprStyleLayers, String> {
        let mut resolved = inherited.clone();
        for (id, override_layer) in &self.layers {
            let layer = resolved
                .layers
                .iter_mut()
                .find(|layer| layer.id == *id)
                .ok_or_else(|| format!("NPR layer override refers to missing layer `{id}`"))?;
            if let Some(enabled) = override_layer.enabled {
                layer.enabled = enabled;
            }
            if let Some(opacity) = override_layer.opacity {
                layer.opacity = opacity;
            }
            if let Some(blend) = override_layer.blend {
                layer.blend = blend;
            }
            if let Some(color_source) = override_layer.color_source {
                layer.color_source = color_source;
            }
            if let Some(tool) = override_layer.tool {
                layer.tool = tool;
            }
            if let Some(paint) = override_layer.paint {
                layer.paint = paint;
            }
        }
        if let Some(order) = &self.order {
            if order.len() != resolved.layers.len()
                || order.iter().collect::<BTreeSet<_>>().len() != order.len()
            {
                return Err("NPR layer order must list each inherited layer exactly once".into());
            }
            let mut by_id = resolved
                .layers
                .into_iter()
                .map(|layer| (layer.id.clone(), layer))
                .collect::<BTreeMap<_, _>>();
            resolved.layers = order
                .iter()
                .map(|id| {
                    by_id
                        .remove(id)
                        .ok_or_else(|| format!("NPR layer order refers to missing layer `{id}`"))
                })
                .collect::<Result<_, _>>()?;
        }
        resolved.validate()?;
        Ok(resolved)
    }

    /// Produces the smallest override that makes `resolved` render like the
    /// supplied stack while continuing to inherit untouched properties.
    pub fn from_resolved(
        inherited: &NprStyleLayers,
        resolved: &NprStyleLayers,
    ) -> Result<Self, String> {
        inherited.validate()?;
        resolved.validate()?;
        let inherited_ids = inherited
            .layers
            .iter()
            .map(|layer| layer.id.as_str())
            .collect::<BTreeSet<_>>();
        let resolved_ids = resolved
            .layers
            .iter()
            .map(|layer| layer.id.as_str())
            .collect::<BTreeSet<_>>();
        if inherited_ids != resolved_ids {
            return Err(
                "NPR layer overrides cannot add or remove inherited layer identities".into(),
            );
        }
        let mut overrides = Self::default();
        for layer in &resolved.layers {
            let parent = inherited.layer(&layer.id).expect("identities were checked");
            let override_layer = NprStyleLayerOverride {
                enabled: (layer.enabled != parent.enabled).then_some(layer.enabled),
                opacity: (layer.opacity != parent.opacity).then_some(layer.opacity),
                blend: (layer.blend != parent.blend).then_some(layer.blend),
                color_source: (layer.color_source != parent.color_source)
                    .then_some(layer.color_source),
                tool: (layer.tool != parent.tool).then_some(layer.tool),
                paint: (layer.paint != parent.paint).then_some(layer.paint),
            };
            if override_layer != NprStyleLayerOverride::default() {
                overrides.layers.insert(layer.id.clone(), override_layer);
            }
        }
        let inherited_order = inherited
            .layers
            .iter()
            .map(|layer| &layer.id)
            .collect::<Vec<_>>();
        let resolved_order = resolved
            .layers
            .iter()
            .map(|layer| &layer.id)
            .collect::<Vec<_>>();
        if inherited_order != resolved_order {
            overrides.order = Some(
                resolved
                    .layers
                    .iter()
                    .map(|layer| layer.id.clone())
                    .collect(),
            );
        }
        Ok(overrides)
    }
}

fn enabled() -> bool {
    true
}
fn one() -> f32 {
    1.0
}

/// Ordered layers of a compositional NPR style. Order is authorial data: a
/// renderer must preserve it instead of grouping by a backend pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprStyleLayers {
    pub layers: Vec<NprStyleLayer>,
}
/// Public Drawing Studio names for the neutral layer contracts.
pub type BrushLayer = NprStyleLayer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StylePreset {
    pub id: String,
    pub layers: NprStyleLayers,
    pub palette: BTreeMap<String, Vec4>,
    pub paper: NprBackgroundSettings,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NprBackgroundSettings {
    pub color: Vec4,
    pub grain: f32,
    pub tooth: f32,
    pub seed: u64,
}
impl Default for NprBackgroundSettings {
    fn default() -> Self {
        Self {
            color: Vec4::ONE,
            grain: 0.0,
            tooth: 0.0,
            seed: 0,
        }
    }
}
impl Default for StylePreset {
    fn default() -> Self {
        Self {
            id: "drawing-studio".into(),
            layers: NprStyleLayers::default(),
            palette: BTreeMap::new(),
            paper: NprBackgroundSettings::default(),
        }
    }
}
impl Default for NprStyleLayers {
    fn default() -> Self {
        let brush = |id: &str| {
            Some(BrushInstance {
                brush: crate::BrushReference {
                    id: id.into(),
                    version: 1,
                },
                ..BrushInstance::default()
            })
        };
        Self {
            layers: vec![
                NprStyleLayer {
                    id: "paper".into(),
                    label: "Paper".into(),
                    source: NprGeometrySource::Paper,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                    hatch: None,
                    brush: None,
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
                NprStyleLayer {
                    id: "underpainting".into(),
                    label: "Underpainting".into(),
                    source: NprGeometrySource::Wash,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: Some(NprPaintMedium::default()),
                    hatch: None,
                    brush: brush("watercolour-wash"),
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
                NprStyleLayer {
                    id: "fill".into(),
                    label: "Flat fill".into(),
                    source: NprGeometrySource::FlatFill,
                    // Preserve the existing painterly default. Authors can
                    // explicitly put crisp three-band regions over the wash.
                    enabled: false,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                    hatch: None,
                    brush: brush("flat-fill"),
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
                NprStyleLayer {
                    id: "hatching".into(),
                    label: "Hatching".into(),
                    source: NprGeometrySource::ShadowHatch,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Multiply,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: Some(StrokeTool::Pencil),
                    paint: None,
                    hatch: Some(NprHatchSettings::default()),
                    brush: brush("surface-hatch"),
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
                NprStyleLayer {
                    id: "form-lines".into(),
                    label: "Form lines".into(),
                    source: NprGeometrySource::FormLines,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: Some(StrokeTool::Fineliner),
                    paint: None,
                    hatch: None,
                    brush: brush("ink-liner"),
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
                NprStyleLayer {
                    id: "contours".into(),
                    label: "Contours".into(),
                    source: NprGeometrySource::Silhouette,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                    hatch: None,
                    brush: brush("ink-liner"),
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
                NprStyleLayer {
                    id: "creases".into(),
                    label: "Creases".into(),
                    source: NprGeometrySource::Creases,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                    hatch: None,
                    brush: brush("graphite-study"),
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
                NprStyleLayer {
                    id: "construction".into(),
                    label: "Construction".into(),
                    source: NprGeometrySource::Construction,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: Some(StrokeTool::Pencil),
                    paint: None,
                    hatch: None,
                    brush: brush("graphite-study"),
                    target: GeometryTarget::All,
                    mask: CoverageMask::None,
                },
            ],
        }
    }
}

impl NprStyleLayers {
    pub fn validate(&self) -> Result<(), String> {
        if self.layers.is_empty() || self.layers.len() > 32 {
            return Err("an NPR style must contain 1..=32 layers".into());
        }
        let mut ids = BTreeSet::new();
        for layer in &self.layers {
            if layer.id.is_empty()
                || layer.id.len() > 80
                || !layer
                    .id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Err(format!("invalid NPR layer id `{}`", layer.id));
            }
            if layer.label.trim().is_empty() || layer.label.len() > 80 {
                return Err(format!("invalid NPR layer label for `{}`", layer.id));
            }
            if !ids.insert(&layer.id) {
                return Err(format!("duplicate NPR layer id `{}`", layer.id));
            }
            if !layer.opacity.is_finite() || !(0.0..=1.0).contains(&layer.opacity) {
                return Err(format!("invalid opacity for NPR layer `{}`", layer.id));
            }
            if let NprLayerColorSource::Constant(color) = layer.color_source
                && (!color.is_finite() || color.min_element() < 0.0 || color.max_element() > 1.0)
            {
                return Err(format!(
                    "invalid constant color for NPR layer `{}`",
                    layer.id
                ));
            }
            if let Some(paint) = layer.paint
                && (!paint.wash.is_finite()
                    || !(0.0..=2.0).contains(&paint.wash)
                    || !paint.granulation.is_finite()
                    || !(0.0..=1.0).contains(&paint.granulation))
            {
                return Err(format!("invalid paint medium for NPR layer `{}`", layer.id));
            }
            if let Some(hatch) = layer.hatch
                && (layer.source != NprGeometrySource::ShadowHatch
                    || !hatch.density.is_finite()
                    || !(0.0..=1.0).contains(&hatch.density)
                    || !hatch.spacing.is_finite()
                    || !(0.25..=4.0).contains(&hatch.spacing))
            {
                return Err(format!(
                    "invalid hatch settings for NPR layer `{}`",
                    layer.id
                ));
            }
            if layer.paint.is_some()
                && !matches!(
                    layer.source,
                    NprGeometrySource::Wash | NprGeometrySource::FlatFill
                )
            {
                return Err(format!(
                    "paint medium is invalid for NPR layer `{}`",
                    layer.id
                ));
            }
            if layer.source == NprGeometrySource::Paper
                && (layer.brush.is_some() || layer.tool.is_some() || layer.paint.is_some())
            {
                return Err(format!(
                    "paper layer `{}` cannot declare a brush, tool, or paint medium",
                    layer.id
                ));
            }
            if let Some(brush) = &layer.brush {
                if brush.brush.id.is_empty() || brush.brush.version == 0 {
                    return Err(format!(
                        "invalid brush reference for NPR layer `{}`",
                        layer.id
                    ));
                }
            }
            layer.mask.validate()?;
            if let GeometryTarget::Objects(objects) = &layer.target {
                if objects.iter().any(|id| id.is_empty()) {
                    return Err(format!(
                        "invalid geometry target for NPR layer `{}`",
                        layer.id
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn layer(&self, id: &str) -> Option<&NprStyleLayer> {
        self.layers.iter().find(|layer| layer.id == id)
    }

    pub fn layer_mut(&mut self, id: &str) -> Option<&mut NprStyleLayer> {
        self.layers.iter_mut().find(|layer| layer.id == id)
    }

    /// Resolves every pinned brush reference before extraction. Missing
    /// versions and an application mismatch are authored-document errors, not
    /// an invitation for the renderer to choose a fallback brush.
    pub fn validate_brushes(&self, library: &BrushLibrary) -> Result<(), String> {
        for layer in &self.layers {
            let Some(instance) = &layer.brush else {
                continue;
            };
            let brush = library.resolve(&instance.brush)?;
            let application = match layer.source {
                NprGeometrySource::Wash | NprGeometrySource::FlatFill => BrushApplication::Surface,
                NprGeometrySource::Paper => continue,
                NprGeometrySource::ShadowHatch
                | NprGeometrySource::Silhouette
                | NprGeometrySource::Creases
                | NprGeometrySource::FormLines
                | NprGeometrySource::Construction => BrushApplication::Stroke,
            };
            if !brush.applications.contains(&application) {
                return Err(format!(
                    "brush {}@{} is not compatible with layer `{}` ({:?})",
                    brush.id, brush.version, layer.id, application
                ));
            }
        }
        Ok(())
    }

    pub fn diagnostics(
        &self,
        source: &NprRenderPacket,
        output: &NprRenderPacket,
        extraction_micros: u64,
        tessellation_micros: u64,
    ) -> BTreeMap<String, NprLayerDiagnostics> {
        self.layers
            .iter()
            .map(|layer| {
                let surface_triangles = match layer.source {
                    NprGeometrySource::Wash => source.underpainting.len(),
                    NprGeometrySource::FlatFill => source.fills.len(),
                    NprGeometrySource::Paper => 1,
                    _ => 0,
                };
                let source_marks = source
                    .strokes
                    .iter()
                    .filter(|stroke| stroke_source(stroke) == layer.source)
                    .count();
                let generated_marks = output
                    .strokes
                    .iter()
                    .filter(|stroke| stroke.layer_id.as_deref() == Some(layer.id.as_str()))
                    .count();
                let generated_triangles = match layer.source {
                    NprGeometrySource::Wash => output
                        .underpainting
                        .iter()
                        .filter(|triangle| triangle.layer_id.as_deref() == Some(layer.id.as_str()))
                        .count(),
                    NprGeometrySource::FlatFill => output
                        .fills
                        .iter()
                        .filter(|triangle| triangle.layer_id.as_deref() == Some(layer.id.as_str()))
                        .count(),
                    NprGeometrySource::Paper => 2,
                    _ => 0,
                };
                let mask_coverage = if matches!(&layer.mask, CoverageMask::None) {
                    1.0
                } else {
                    layer.mask.evaluate(0.5, 0.5, [0.0, 0.0, 1.0], 0.5)
                };
                let source_geometry = source_marks + surface_triangles;
                let no_effect_reason = if !layer.enabled {
                    Some(NprLayerNoEffectReason::Disabled)
                } else if layer.opacity <= 0.0 {
                    Some(NprLayerNoEffectReason::ZeroOpacity)
                } else if source_geometry == 0 {
                    Some(NprLayerNoEffectReason::NoSourceGeometry)
                } else if mask_coverage <= 0.0 {
                    Some(NprLayerNoEffectReason::MaskExcludesAll)
                } else if layer.hatch.is_some_and(|hatch| hatch.density <= 0.0) {
                    Some(NprLayerNoEffectReason::HatchSelectionExcludesAll)
                } else {
                    None
                };
                (
                    layer.id.clone(),
                    NprLayerDiagnostics {
                        source_geometry,
                        generated_marks,
                        generated_triangles,
                        mask_coverage,
                        extraction_micros,
                        tessellation_micros,
                        no_effect_reason,
                    },
                )
            })
            .collect()
    }

    /// Every enabled layer which declares this source geometry.  This is the
    /// important distinction between a source and a compositing layer: two
    /// hatch layers share extracted hatch paths but produce two independent
    /// contributions with their own brush, mask, colour and opacity.
    pub fn stroke_layers(
        &self,
        stroke: &TessellatedStroke,
    ) -> impl Iterator<Item = &NprStyleLayer> {
        let source = stroke_source(stroke);
        self.layers
            .iter()
            .filter(move |layer| layer.enabled && layer.source == source)
    }

    /// Applies an explicitly selected layer tool to already extracted gesture
    /// geometry. This reconstructs strip centres and rescales body plus round
    /// caps in the NPR domain, before a render backend receives the packet.
    pub fn apply_tools(&self, packet: &mut NprRenderPacket, style: ComicInk) {
        self.apply_tools_with_library(packet, style, None)
            .expect("layer brush overrides are validated before extraction");
    }

    /// Applies layer tools while resolving defaults from the pinned brush
    /// library. Local instance fields always win over definition defaults.
    pub fn apply_tools_with_library(
        &self,
        packet: &mut NprRenderPacket,
        style: ComicInk,
        library: Option<&BrushLibrary>,
    ) -> Result<(), String> {
        let library = library.filter(|library| !library.brushes.is_empty());
        let source_strokes = std::mem::take(&mut packet.strokes);
        let mut contributions = Vec::new();
        for stroke in source_strokes {
            for layer in self.stroke_layers(&stroke) {
                if let Some(hatch) = layer.hatch {
                    let density = (hatch.density / hatch.spacing).clamp(0.0, 1.0);
                    let layer_hash = layer.id.bytes().fold(0u64, |hash, byte| {
                        hash.wrapping_mul(1099511628211) ^ u64::from(byte)
                    });
                    let sample = ((u64::from(stroke.id) ^ layer_hash) % 10_000) as f32 / 10_000.0;
                    if sample > density {
                        continue;
                    }
                }
                let mut stroke = stroke.clone();
                stroke.layer_id = Some(layer.id.clone());
                if let Some(tool) = layer.tool
                    && tool != style.tool
                {
                    retarget_stroke_tool(&mut stroke, style, tool);
                }
                let definition = match (layer.brush.as_ref(), library) {
                    (Some(brush), Some(library)) => Some(library.resolve(&brush.brush)?),
                    _ => None,
                };
                let width = layer
                    .brush
                    .as_ref()
                    .and_then(|brush| brush.width)
                    .or_else(|| definition.map(|brush| brush.width));
                if let Some(width) = width {
                    let factor = (width / style.outline_width.max(0.001)).clamp(0.05, 20.0);
                    scale_stroke_width(&mut stroke, factor);
                }
                let taper = layer
                    .brush
                    .as_ref()
                    .and_then(|brush| brush.taper)
                    .or_else(|| definition.map(|brush| brush.taper));
                if let Some(taper) = taper {
                    let last = stroke.vertices.len().saturating_sub(1).max(1) as f32;
                    for (index, vertex) in stroke.vertices.iter_mut().enumerate() {
                        let edge = ((index as f32 / last) * 2.0 - 1.0).abs();
                        vertex.coverage *= (1.0 - taper.clamp(0.0, 1.0) * edge).max(0.002);
                    }
                }
                let softness = layer
                    .brush
                    .as_ref()
                    .and_then(|brush| brush.softness)
                    .or_else(|| definition.map(|brush| brush.softness));
                if let Some(softness) = softness {
                    for vertex in &mut stroke.vertices {
                        vertex.edge_softness = softness.clamp(0.0, 1.0);
                    }
                }
                if let Some(brush) = &layer.brush {
                    let seed = brush
                        .seed
                        .or_else(|| definition.map(|definition| definition.seed))
                        .unwrap_or(0);
                    if let Some(irregularity) = brush
                        .irregularity
                        .or_else(|| definition.map(|definition| definition.irregularity))
                    {
                        let amount = irregularity.clamp(0.0, 1.0);
                        for (index, vertex) in stroke.vertices.iter_mut().enumerate() {
                            let phase =
                                (seed as f32 * 0.000_013 + index as f32 * 1.618_033_9).sin() * 0.5
                                    + 0.5;
                            vertex.coverage = (vertex.coverage
                                * (1.0 - amount * (0.25 + phase * 0.5)))
                                .clamp(0.002, 1.0);
                        }
                    }
                    if let Some(dryness) = brush
                        .dryness
                        .or_else(|| definition.map(|definition| definition.dryness))
                    {
                        let dryness = dryness.clamp(0.0, 1.0);
                        for vertex in &mut stroke.vertices {
                            vertex.dryness = dryness;
                        }
                    }
                }
                contributions.push(stroke);
            }
        }
        packet.strokes = contributions;
        let source_fills = std::mem::take(&mut packet.fills);
        let mut fill_contributions = source_fills.clone();
        fill_contributions.extend(
            self.layers
                .iter()
                .filter(|layer| layer.enabled && layer.source == NprGeometrySource::FlatFill)
                .flat_map(|layer| {
                    source_fills.iter().cloned().map(move |mut triangle| {
                        triangle.layer_id = Some(layer.id.clone());
                        triangle
                    })
                }),
        );
        packet.fills = fill_contributions;
        let source_paint = std::mem::take(&mut packet.underpainting);
        let mut paint_contributions = source_paint.clone();
        paint_contributions.extend(
            self.layers
                .iter()
                .filter(|layer| layer.enabled && layer.source == NprGeometrySource::Wash)
                .flat_map(|layer| {
                    source_paint.iter().cloned().map(move |mut triangle| {
                        triangle.layer_id = Some(layer.id.clone());
                        triangle
                    })
                }),
        );
        packet.underpainting = paint_contributions;
        Ok(())
    }
}

fn stroke_source(stroke: &TessellatedStroke) -> NprGeometrySource {
    match stroke.role {
        StrokeRole::Tone => NprGeometrySource::ShadowHatch,
        StrokeRole::FormLine => NprGeometrySource::FormLines,
        StrokeRole::Construction => NprGeometrySource::Construction,
        StrokeRole::Feature if stroke.class == FeatureClass::Crease => NprGeometrySource::Creases,
        StrokeRole::Feature => NprGeometrySource::Silhouette,
    }
}

fn scale_stroke_width(stroke: &mut TessellatedStroke, factor: f32) {
    if !factor.is_finite() || (factor - 1.0).abs() < 1e-5 {
        return;
    }
    for vertex in &mut stroke.vertices {
        vertex.width *= factor;
    }
    // Body vertices are emitted as left/right pairs around a shared centre.
    // Rescale those pairs and round-cap rings around their local centres so a
    // brush width override changes the visible strip, not just its metadata.
    for pair in stroke.vertices.chunks_exact_mut(2) {
        if pair[0].edge.abs() > 0.5 && pair[1].edge.abs() > 0.5 {
            let centre = (pair[0].position + pair[1].position) * 0.5;
            pair[0].position = centre + (pair[0].position - centre) * factor;
            pair[1].position = centre + (pair[1].position - centre) * factor;
        }
    }
    let mut index = 0;
    while index < stroke.vertices.len() {
        if stroke.vertices[index].edge.abs() < 0.5 {
            let centre = stroke.vertices[index].position;
            index += 1;
            while index < stroke.vertices.len() && stroke.vertices[index].edge.abs() >= 0.5 {
                stroke.vertices[index].position =
                    centre + (stroke.vertices[index].position - centre) * factor;
                index += 1;
            }
        } else {
            index += 1;
        }
    }
}

fn retarget_stroke_tool(stroke: &mut TessellatedStroke, style: ComicInk, target_tool: StrokeTool) {
    // Strip body vertices are paired (-1, +1). A round cap starts with its
    // centre (`edge == 0`), so no renderer-facing bookkeeping is required.
    let body_len = stroke
        .vertices
        .iter()
        .position(|vertex| vertex.edge.abs() < 1e-6)
        .unwrap_or(stroke.vertices.len());
    let pair_len = body_len - body_len % 2;
    if pair_len < 4 {
        return;
    }
    let centres = (0..pair_len)
        .step_by(2)
        .map(|index| (stroke.vertices[index].position + stroke.vertices[index + 1].position) * 0.5)
        .collect::<Vec<_>>();
    let mut endpoint_scales = [1.0, 1.0];
    for pair in 0..centres.len() {
        let index = pair * 2;
        let previous = centres[pair.saturating_sub(1)];
        let next = centres[(pair + 1).min(centres.len() - 1)];
        let tangent = next - previous;
        let tangent_angle = tangent.y.atan2(tangent.x) - style.nib_angle.to_radians();
        let pressure = stroke.vertices[index].pressure.clamp(0.0, 1.0);
        let source = style
            .tool
            .response(pressure, tangent_angle, style.tool_hardness);
        let target = target_tool.response(pressure, tangent_angle, style.tool_hardness);
        let source_aspect = nib_aspect(style.tool, style.nib_aspect);
        let target_aspect = nib_aspect(target_tool, style.nib_aspect);
        let scale = (target.width_scale * target.pressure_width * target_aspect
            / (source.width_scale * source.pressure_width * source_aspect).max(1e-5))
        .clamp(0.2, 4.0);
        if pair == 0 {
            endpoint_scales[0] = scale;
        }
        if pair + 1 == centres.len() {
            endpoint_scales[1] = scale;
        }
        let alpha_scale =
            (target.pressure_alpha / source.pressure_alpha.max(1e-5)).clamp(0.15, 4.0);
        for vertex in &mut stroke.vertices[index..index + 2] {
            vertex.position = centres[pair] + (vertex.position - centres[pair]) * scale;
            vertex.width *= scale;
            vertex.coverage = (vertex.coverage * alpha_scale).clamp(0.002, 1.0);
            vertex.edge_softness = target.edge_softness;
        }
    }
    // Preserve circular caps while assigning their matching endpoint radius.
    let mut cap_index = pair_len;
    for scale in endpoint_scales {
        let Some(center) = stroke.vertices.get(cap_index).map(|vertex| vertex.position) else {
            break;
        };
        let mut end = cap_index + 1;
        while end < stroke.vertices.len() && stroke.vertices[end].edge.abs() >= 1e-6 {
            end += 1;
        }
        for vertex in &mut stroke.vertices[cap_index..end] {
            vertex.position = center + (vertex.position - center) * scale;
            vertex.width *= scale;
        }
        cap_index = end;
    }
}

fn nib_aspect(tool: StrokeTool, nib_aspect: f32) -> f32 {
    if matches!(tool, StrokeTool::Nib) {
        1.0 + nib_aspect.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BrushDefinition;

    #[test]
    fn default_stack_has_stable_ordered_drawing_layers() {
        let layers = NprStyleLayers::default();
        layers.validate().unwrap();
        assert_eq!(
            layers
                .layers
                .iter()
                .map(|layer| layer.id.as_str())
                .collect::<Vec<_>>(),
            [
                "paper",
                "underpainting",
                "fill",
                "hatching",
                "form-lines",
                "contours",
                "creases",
                "construction"
            ]
        );
        assert_eq!(
            layers.layer("hatching").unwrap().blend,
            NprBlendMode::Multiply
        );
    }

    #[test]
    fn validation_rejects_duplicate_identity_and_invalid_opacity() {
        let mut layers = NprStyleLayers::default();
        layers.layers[1].id = "paper".into();
        assert!(layers.validate().unwrap_err().contains("duplicate"));
        layers.layers[1].id = "underpainting".into();
        layers.layers[1].opacity = 1.5;
        assert!(layers.validate().unwrap_err().contains("opacity"));
    }

    #[test]
    fn disabled_layer_suppresses_only_its_declared_mark_kind() {
        let mut layers = NprStyleLayers::default();
        layers.layer("hatching").unwrap();
        layers
            .layers
            .iter_mut()
            .find(|layer| layer.id == "hatching")
            .unwrap()
            .enabled = false;
        let tone = TessellatedStroke {
            role: StrokeRole::Tone,
            ..Default::default()
        };
        let contour = TessellatedStroke {
            role: StrokeRole::Feature,
            class: FeatureClass::Silhouette,
            ..Default::default()
        };
        assert!(layers.stroke_layers(&tone).next().is_none());
        assert_eq!(
            layers.stroke_layers(&contour).next().unwrap().id,
            "contours"
        );
    }

    #[test]
    fn crease_and_contour_layers_can_be_reordered_without_reclassifying_marks() {
        let mut layers = NprStyleLayers::default();
        let contours = layers
            .layers
            .iter()
            .position(|layer| layer.id == "contours")
            .unwrap();
        let creases = layers
            .layers
            .iter()
            .position(|layer| layer.id == "creases")
            .unwrap();
        layers.layers.swap(contours, creases);
        let crease = TessellatedStroke {
            role: StrokeRole::Feature,
            class: FeatureClass::Crease,
            ..Default::default()
        };
        let contour = TessellatedStroke {
            role: StrokeRole::Feature,
            class: FeatureClass::Silhouette,
            ..Default::default()
        };
        assert_eq!(layers.stroke_layers(&crease).next().unwrap().id, "creases");
        assert_eq!(
            layers.stroke_layers(&contour).next().unwrap().id,
            "contours"
        );
    }

    #[test]
    fn same_source_layers_expand_into_independent_brush_contributions() {
        let mut layers = NprStyleLayers::default();
        let mut second = layers.layer("hatching").unwrap().clone();
        second.id = "hatching-soft".into();
        second.opacity = 0.35;
        second.brush.as_mut().unwrap().width = Some(3.0);
        layers.layers.push(second);
        let source = TessellatedStroke {
            role: StrokeRole::Tone,
            ..Default::default()
        };
        let mut packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![],
            fills: vec![],
            strokes: vec![source],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        layers.apply_tools(&mut packet, ComicInk::default());
        assert_eq!(packet.strokes.len(), 2);
        assert_eq!(packet.strokes[0].layer_id.as_deref(), Some("hatching"));
        assert_eq!(packet.strokes[1].layer_id.as_deref(), Some("hatching-soft"));
    }

    #[test]
    fn same_surface_source_expands_into_independent_layer_contributions() {
        let mut layers = NprStyleLayers::default();
        layers.layer_mut("fill").unwrap().enabled = true;
        layers.layer_mut("underpainting").unwrap().enabled = true;
        let mut second_fill = layers.layer("fill").unwrap().clone();
        second_fill.id = "fill-soft".into();
        layers.layers.push(second_fill);
        let mut second_wash = layers.layer("underpainting").unwrap().clone();
        second_wash.id = "wash-soft".into();
        layers.layers.push(second_wash);
        let source_packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![crate::NprPaintTriangle {
                positions: [glam::Vec2::ZERO, glam::Vec2::X, glam::Vec2::Y],
                color: Vec4::ONE,
                depths: [0.1, 0.2, 0.3],
                coverage: 1.0,
                layer_id: None,
            }],
            fills: vec![crate::NprFillTriangle {
                positions: [glam::Vec2::ZERO, glam::Vec2::X, glam::Vec2::Y],
                color: Vec4::ONE,
                depths: [0.1, 0.2, 0.3],
                layer_id: None,
            }],
            strokes: vec![],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        let mut output = source_packet.clone();
        layers.apply_tools(&mut output, ComicInk::default());
        assert_eq!(
            output
                .fills
                .iter()
                .filter(|triangle| triangle.layer_id.is_some())
                .count(),
            2
        );
        assert_eq!(
            output
                .underpainting
                .iter()
                .filter(|triangle| triangle.layer_id.is_some())
                .count(),
            2
        );
        assert!(
            output
                .fills
                .iter()
                .any(|triangle| triangle.layer_id.as_deref() == Some("fill"))
        );
        assert!(
            output
                .fills
                .iter()
                .any(|triangle| triangle.layer_id.as_deref() == Some("fill-soft"))
        );
        assert!(
            output
                .underpainting
                .iter()
                .any(|triangle| triangle.layer_id.as_deref() == Some("underpainting"))
        );
        assert!(
            output
                .underpainting
                .iter()
                .any(|triangle| triangle.layer_id.as_deref() == Some("wash-soft"))
        );
        let diagnostics = layers.diagnostics(&source_packet, &output, 0, 0);
        assert_eq!(diagnostics["fill"].generated_triangles, 1);
        assert_eq!(diagnostics["fill-soft"].generated_triangles, 1);
        assert_eq!(diagnostics["underpainting"].generated_triangles, 1);
        assert_eq!(diagnostics["wash-soft"].generated_triangles, 1);
    }

    #[test]
    fn diagnostics_identify_source_coverage_and_no_effect_reason() {
        let mut layers = NprStyleLayers::default();
        let source = TessellatedStroke {
            role: StrokeRole::Tone,
            ..Default::default()
        };
        let source_packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![],
            fills: vec![],
            strokes: vec![source],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        let mut output = source_packet.clone();
        layers.apply_tools(&mut output, ComicInk::default());
        let report = layers.diagnostics(&source_packet, &output, 12, 7);
        let hatch = &report["hatching"];
        assert_eq!(hatch.source_geometry, 1);
        assert_eq!(hatch.generated_marks, 1);
        assert_eq!(hatch.extraction_micros, 12);
        assert_eq!(hatch.tessellation_micros, 7);
        assert_eq!(hatch.no_effect_reason, None);

        let hatch = layers.layer_mut("hatching").unwrap();
        hatch.mask = CoverageMask::ToneRange {
            min: 0.8,
            max: 1.0,
            invert: false,
        };
        assert_eq!(
            layers.diagnostics(&source_packet, &output, 0, 0)["hatching"].no_effect_reason,
            Some(NprLayerNoEffectReason::MaskExcludesAll)
        );
    }

    #[test]
    fn brush_instance_taper_and_softness_change_only_its_layer_contribution() {
        let mut layers = NprStyleLayers::default();
        let contour = layers.layer_mut("contours").unwrap();
        contour.brush.as_mut().unwrap().taper = Some(0.8);
        contour.brush.as_mut().unwrap().softness = Some(0.73);
        let stroke = crate::tessellate_segment(
            1,
            FeatureClass::Silhouette,
            (glam::Vec2::ZERO, glam::Vec2::new(80.0, 0.0)),
            ComicInk::default(),
            4,
        );
        let mut packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![],
            fills: vec![],
            strokes: vec![stroke],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        layers.apply_tools(&mut packet, ComicInk::default());
        let contour = packet
            .strokes
            .iter()
            .find(|stroke| stroke.layer_id.as_deref() == Some("contours"))
            .unwrap();
        assert!(contour.vertices.iter().any(|vertex| vertex.coverage < 0.5));
        assert!(
            contour
                .vertices
                .iter()
                .all(|vertex| vertex.edge_softness == 0.73)
        );
    }

    #[test]
    fn hatch_density_filters_only_the_owning_layer_contribution() {
        let mut layers = NprStyleLayers::default();
        layers.layer_mut("hatching").unwrap().hatch = Some(NprHatchSettings {
            density: 0.0,
            spacing: 1.0,
        });
        let source = TessellatedStroke {
            id: 7,
            role: StrokeRole::Tone,
            ..Default::default()
        };
        let source_packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![],
            fills: vec![],
            strokes: vec![source],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        let mut output = source_packet.clone();
        layers.apply_tools(&mut output, ComicInk::default());
        assert!(output.strokes.is_empty());
        assert_eq!(
            layers.diagnostics(&source_packet, &output, 0, 0)["hatching"].no_effect_reason,
            Some(NprLayerNoEffectReason::HatchSelectionExcludesAll)
        );
    }

    #[test]
    fn sparse_override_round_trips_changed_layer_properties_and_order() {
        let parent = NprStyleLayers::default();
        let mut child = parent.clone();
        let hatching = child
            .layers
            .iter_mut()
            .find(|layer| layer.id == "hatching")
            .unwrap();
        hatching.opacity = 0.35;
        hatching.blend = NprBlendMode::Screen;
        hatching.color_source = NprLayerColorSource::Constant(Vec4::new(0.1, 0.2, 0.3, 1.0));
        hatching.tool = Some(StrokeTool::Brush);
        child
            .layers
            .iter_mut()
            .find(|layer| layer.id == "underpainting")
            .unwrap()
            .paint = Some(NprPaintMedium {
            wash: 0.7,
            granulation: 0.3,
        });
        let hatching_index = child
            .layers
            .iter()
            .position(|layer| layer.id == "hatching")
            .unwrap();
        let layer = child.layers.remove(hatching_index);
        child.layers.insert(1, layer);

        let overrides = NprStyleLayerOverrides::from_resolved(&parent, &child).unwrap();
        assert!(overrides.layers.contains_key("hatching"));
        assert!(overrides.order.is_some());
        assert_eq!(overrides.resolve(&parent).unwrap(), child);
    }

    #[test]
    fn sparse_override_rejects_missing_parent_layer_instead_of_falling_back() {
        let mut overrides = NprStyleLayerOverrides::default();
        overrides.layers.insert(
            "missing".into(),
            NprStyleLayerOverride {
                opacity: Some(0.5),
                ..Default::default()
            },
        );
        assert!(
            overrides
                .resolve(&NprStyleLayers::default())
                .unwrap_err()
                .contains("missing layer")
        );
    }

    #[test]
    fn layer_tool_retargets_body_width_and_material_before_backend_execution() {
        let style = ComicInk {
            tool: StrokeTool::Fineliner,
            outline_width: 8.0,
            tool_hardness: 0.4,
            ..Default::default()
        };
        let mut stroke = crate::tessellate_segment(
            7,
            FeatureClass::Silhouette,
            (glam::Vec2::new(20.0, 20.0), glam::Vec2::new(140.0, 30.0)),
            style,
            19,
        );
        let before = stroke.clone();
        retarget_stroke_tool(&mut stroke, style, StrokeTool::Pencil);

        assert_ne!(stroke.vertices, before.vertices);
        assert!(
            stroke
                .vertices
                .iter()
                .all(|vertex| vertex.width.is_finite())
        );
        assert!(
            stroke
                .vertices
                .iter()
                .any(|vertex| vertex.edge_softness > 0.1)
        );
        assert_eq!(stroke.indices, before.indices);
    }

    #[test]
    fn brush_width_override_changes_tessellated_strip_geometry() {
        let style = ComicInk::default();
        let mut layers = NprStyleLayers::default();
        layers.layer_mut("contours").unwrap().brush = Some(BrushInstance {
            brush: crate::BrushReference {
                id: "ink-liner".into(),
                version: 1,
            },
            width: Some(style.outline_width * 2.0),
            irregularity: Some(0.8),
            dryness: Some(0.9),
            seed: Some(17),
            ..BrushInstance::default()
        });
        let stroke = crate::tessellate_segment(
            3,
            FeatureClass::Silhouette,
            (glam::Vec2::new(20.0, 20.0), glam::Vec2::new(120.0, 30.0)),
            style,
            11,
        );
        let before = stroke.vertices.clone();
        let mut packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![],
            fills: vec![],
            strokes: vec![stroke],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        layers.apply_tools(&mut packet, style);
        assert!(
            packet.strokes[0]
                .vertices
                .iter()
                .zip(before)
                .any(|(after, before)| { (after.position - before.position).length() > 1e-4 })
        );
        assert!(
            packet.strokes[0]
                .vertices
                .iter()
                .all(|vertex| vertex.dryness == 0.9)
        );
        assert!(
            packet.strokes[0]
                .vertices
                .iter()
                .any(|vertex| vertex.coverage < 1.0)
        );
    }

    #[test]
    fn pinned_brush_definition_supplies_default_width_and_dryness() {
        let style = ComicInk::default();
        let layers = NprStyleLayers::default();
        let mut library = BrushLibrary::default();
        library
            .add_version(BrushDefinition {
                id: "ink-liner".into(),
                width: style.outline_width * 2.0,
                dryness: 0.85,
                ..BrushDefinition::default()
            })
            .unwrap();
        let stroke = crate::tessellate_segment(
            3,
            FeatureClass::Silhouette,
            (glam::Vec2::new(20.0, 20.0), glam::Vec2::new(120.0, 30.0)),
            style,
            11,
        );
        let before = stroke.vertices.clone();
        let mut packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![],
            fills: vec![],
            strokes: vec![stroke],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        layers
            .apply_tools_with_library(&mut packet, style, Some(&library))
            .unwrap();
        assert!(
            packet.strokes[0]
                .vertices
                .iter()
                .zip(before)
                .any(|(after, before)| { (after.position - before.position).length() > 1e-4 })
        );
        assert!(
            packet.strokes[0]
                .vertices
                .iter()
                .all(|vertex| vertex.dryness == 0.85)
        );
    }

    #[test]
    fn brush_definition_seed_keeps_humanization_deterministic() {
        let style = ComicInk::default();
        let layers = NprStyleLayers::default();
        let mut library = BrushLibrary::default();
        library
            .add_version(BrushDefinition {
                id: "ink-liner".into(),
                irregularity: 0.7,
                seed: 0x1234_5678,
                ..BrushDefinition::default()
            })
            .unwrap();
        let stroke = crate::tessellate_segment(
            3,
            FeatureClass::Silhouette,
            (glam::Vec2::new(20.0, 20.0), glam::Vec2::new(120.0, 30.0)),
            style,
            11,
        );
        let packet = NprRenderPacket {
            occluders: vec![],
            underpainting: vec![],
            fills: vec![],
            strokes: vec![stroke],
            background: Vec4::ZERO,
            debug_view: crate::NprDebugView::Final,
            ink: Vec4::ONE,
            stats: crate::NprRenderStats::default(),
        };
        let mut first = packet.clone();
        let mut second = packet;
        layers
            .apply_tools_with_library(&mut first, style, Some(&library))
            .unwrap();
        layers
            .apply_tools_with_library(&mut second, style, Some(&library))
            .unwrap();
        assert_eq!(first.strokes[0].vertices, second.strokes[0].vertices);
    }
}
