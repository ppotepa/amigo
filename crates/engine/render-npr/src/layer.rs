//! Typed compositing intent for an NPR drawing.
//!
//! These types describe authorial layers before a render backend decides how a
//! pass is executed. They deliberately do not contain WGPU resources, shader
//! names, or inferred material policy.

use crate::{ComicInk, FeatureClass, NprRenderPacket, StrokeRole, StrokeTool, TessellatedStroke};
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NprLayerKind {
    Paper,
    Underpainting,
    Fill,
    Hatching,
    Contours,
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
    Overlay,
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
    pub kind: NprLayerKind,
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

impl Default for NprStyleLayers {
    fn default() -> Self {
        Self {
            layers: vec![
                NprStyleLayer {
                    id: "paper".into(),
                    label: "Paper".into(),
                    kind: NprLayerKind::Paper,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                },
                NprStyleLayer {
                    id: "underpainting".into(),
                    label: "Underpainting".into(),
                    kind: NprLayerKind::Underpainting,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: Some(NprPaintMedium::default()),
                },
                NprStyleLayer {
                    id: "fill".into(),
                    label: "Flat fill".into(),
                    kind: NprLayerKind::Fill,
                    // Preserve the existing painterly default. Authors can
                    // explicitly put crisp three-band regions over the wash.
                    enabled: false,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                },
                NprStyleLayer {
                    id: "hatching".into(),
                    label: "Hatching".into(),
                    kind: NprLayerKind::Hatching,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Multiply,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: Some(StrokeTool::Pencil),
                    paint: None,
                },
                NprStyleLayer {
                    id: "form-lines".into(),
                    label: "Form lines".into(),
                    kind: NprLayerKind::FormLines,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: Some(StrokeTool::Fineliner),
                    paint: None,
                },
                NprStyleLayer {
                    id: "contours".into(),
                    label: "Contours".into(),
                    kind: NprLayerKind::Contours,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                },
                NprStyleLayer {
                    id: "creases".into(),
                    label: "Creases".into(),
                    kind: NprLayerKind::Creases,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: None,
                    paint: None,
                },
                NprStyleLayer {
                    id: "construction".into(),
                    label: "Construction".into(),
                    kind: NprLayerKind::Construction,
                    enabled: true,
                    opacity: 1.0,
                    blend: NprBlendMode::Normal,
                    color_source: NprLayerColorSource::StylePalette,
                    tool: Some(StrokeTool::Pencil),
                    paint: None,
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
            if layer.paint.is_some()
                && !matches!(layer.kind, NprLayerKind::Underpainting | NprLayerKind::Fill)
            {
                return Err(format!(
                    "paint medium is invalid for NPR layer `{}`",
                    layer.id
                ));
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

    /// The active paint layer for the current renderer contract. `Fill` and
    /// `Underpainting` both map to the packet's flat fill geometry until the
    /// packet carries independently generated paint media.
    pub fn fill_layer(&self) -> Option<&NprStyleLayer> {
        self.layers.iter().find(|layer| {
            layer.enabled && matches!(layer.kind, NprLayerKind::Underpainting | NprLayerKind::Fill)
        })
    }

    /// Selects the authored layer responsible for an emitted stroke. A missing
    /// layer deliberately suppresses the mark rather than making a renderer
    /// side guess about a suitable fallback.
    pub fn stroke_layer(&self, stroke: &TessellatedStroke) -> Option<&NprStyleLayer> {
        let kind = match stroke.role {
            StrokeRole::Tone => NprLayerKind::Hatching,
            StrokeRole::FormLine => NprLayerKind::FormLines,
            StrokeRole::Construction => NprLayerKind::Construction,
            StrokeRole::Feature if stroke.class == FeatureClass::Crease => NprLayerKind::Creases,
            StrokeRole::Feature => NprLayerKind::Contours,
        };
        self.layers
            .iter()
            .find(|layer| layer.enabled && layer.kind == kind)
    }

    /// Applies an explicitly selected layer tool to already extracted gesture
    /// geometry. This reconstructs strip centres and rescales body plus round
    /// caps in the NPR domain, before a render backend receives the packet.
    pub fn apply_tools(&self, packet: &mut NprRenderPacket, style: ComicInk) {
        for stroke in &mut packet.strokes {
            let Some(tool) = self.stroke_layer(stroke).and_then(|layer| layer.tool) else {
                continue;
            };
            if tool != style.tool {
                retarget_stroke_tool(stroke, style, tool);
            }
        }
    }

    /// Resolves authored wash coverage into the independent underpainting
    /// channel. Granulation is sampled continuously by the paint material so
    /// it cannot expose source-triangle boundaries.
    pub fn apply_paint_media(&self, packet: &mut NprRenderPacket) {
        let Some(medium) = self
            .layers
            .iter()
            .find(|layer| layer.enabled && layer.kind == NprLayerKind::Underpainting)
            .and_then(|layer| layer.paint)
        else {
            return;
        };
        for triangle in &mut packet.underpainting {
            triangle.coverage = (triangle.coverage * medium.wash).clamp(0.0, 1.0);
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
        assert!(layers.stroke_layer(&tone).is_none());
        assert_eq!(layers.stroke_layer(&contour).unwrap().id, "contours");
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
        assert_eq!(layers.stroke_layer(&crease).unwrap().id, "creases");
        assert_eq!(layers.stroke_layer(&contour).unwrap().id, "contours");
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
}
