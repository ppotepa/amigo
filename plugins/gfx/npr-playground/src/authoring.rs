//! Domain-owned layer authoring operations for Drawing Studio.
//!
//! The companion sends intents, never partially assembled layer stacks.  This
//! module owns stable ids, source defaults, and the invariants that are shared
//! by every host of the playground.

use amigo_render_npr::{
    BrushInstance, BrushLibrary, BrushReference, CoverageMask, GeometryTarget, NprBlendMode,
    NprGeometrySource, NprHatchSettings, NprLayerColorSource, NprPaintMedium, NprStyleLayer,
    NprStyleLayers, StrokeTool,
};
use std::collections::BTreeSet;

pub const MAX_LAYERS: usize = 32;

pub fn source_key(source: NprGeometrySource) -> &'static str {
    source.key()
}

pub fn next_layer_id(layers: &NprStyleLayers, source: NprGeometrySource) -> String {
    let base = source_key(source);
    if layers.layer(base).is_none() {
        return base.into();
    }
    (2..=MAX_LAYERS)
        .map(|number| format!("{base}-{number}"))
        .find(|id| layers.layer(id).is_none())
        .unwrap_or_else(|| format!("{base}-{}", layers.layers.len() + 1))
}

fn default_brush(source: NprGeometrySource) -> Option<BrushInstance> {
    let id = match source {
        NprGeometrySource::Wash => "watercolour-wash",
        NprGeometrySource::FlatFill => "flat-fill",
        NprGeometrySource::ShadowHatch => "surface-hatch",
        NprGeometrySource::Silhouette | NprGeometrySource::FormLines => "ink-liner",
        NprGeometrySource::Creases | NprGeometrySource::Construction => "graphite-study",
        NprGeometrySource::Paper => return None,
    };
    Some(BrushInstance {
        brush: BrushReference {
            id: id.into(),
            version: 1,
        },
        ..BrushInstance::default()
    })
}

/// Creates one layer without cloning an existing layer as a UI template.
pub fn create_layer(
    layers: &NprStyleLayers,
    source: NprGeometrySource,
    label: Option<String>,
    brush: Option<BrushInstance>,
    target: Option<GeometryTarget>,
    mask: Option<CoverageMask>,
) -> Result<NprStyleLayer, String> {
    if source == NprGeometrySource::Paper {
        return Err("Paper is the pinned document layer and cannot be added".into());
    }
    if layers.layers.len() >= MAX_LAYERS {
        return Err("layer limit reached".into());
    }
    let id = next_layer_id(layers, source);
    let surface = matches!(
        source,
        NprGeometrySource::Wash | NprGeometrySource::FlatFill
    );
    let hatch = (source == NprGeometrySource::ShadowHatch).then(NprHatchSettings::default);
    let paint = (source == NprGeometrySource::Wash).then(NprPaintMedium::default);
    let tool = match source {
        NprGeometrySource::ShadowHatch | NprGeometrySource::Construction => {
            Some(StrokeTool::Pencil)
        }
        NprGeometrySource::FormLines => Some(StrokeTool::Fineliner),
        _ => None,
    };
    Ok(NprStyleLayer {
        id,
        label: label
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                source_key(source)
                    .split('-')
                    .map(|word| {
                        let mut chars = word.chars();
                        chars.next().map_or_else(String::new, |first| {
                            first.to_uppercase().collect::<String>() + chars.as_str()
                        })
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
        source,
        enabled: true,
        opacity: 1.0,
        blend: (source == NprGeometrySource::ShadowHatch)
            .then_some(NprBlendMode::Multiply)
            .unwrap_or_default(),
        color_source: NprLayerColorSource::StylePalette,
        tool: if surface { None } else { tool },
        paint,
        hatch,
        line: None,
        brush: brush.or_else(|| default_brush(source)),
        target: target.unwrap_or_default(),
        mask: mask.unwrap_or_default(),
    })
}

pub fn validate_layer_edit(
    layers: &NprStyleLayers,
    library: &BrushLibrary,
    locked: &BTreeSet<String>,
    layer_id: &str,
    candidate: &NprStyleLayer,
) -> Result<(), String> {
    let current = layers
        .layer(layer_id)
        .ok_or_else(|| format!("unknown layer `{layer_id}`"))?;
    if locked.contains(layer_id) && current != candidate {
        return Err(format!("layer is locked: {layer_id}"));
    }
    if candidate.id != layer_id {
        return Err("layer replacement cannot change its stable id".into());
    }
    if current.source != candidate.source {
        return Err("layer source cannot change; add a new layer instead".into());
    }
    if candidate.source == NprGeometrySource::Paper
        && (candidate.brush.is_some() || candidate.tool.is_some() || candidate.paint.is_some())
    {
        return Err("Paper cannot declare a brush, tool, or paint medium".into());
    }
    candidate.mask.validate()?;
    let has_empty_target_id = match &candidate.target {
        GeometryTarget::Objects { objects } => objects.iter().any(|id| id.trim().is_empty()),
        GeometryTarget::SurfaceFeatures { features } => {
            features.iter().any(|id| id.trim().is_empty())
        }
        GeometryTarget::All => false,
    };
    if has_empty_target_id {
        return Err("geometry target contains an empty id".into());
    }
    let mut replacement = layers.clone();
    *replacement
        .layer_mut(layer_id)
        .expect("layer checked above") = candidate.clone();
    replacement.validate()?;
    replacement.validate_brushes(library)?;
    Ok(())
}

/// Capture effective appearance and update references as one history operation.
pub fn save_appearance(
    layers: &mut NprStyleLayers,
    library: &mut BrushLibrary,
    locked: &BTreeSet<String>,
    layer_id: &str,
    value: NprStyleLayer,
    name: String,
    update_matching: bool,
) -> Result<(), String> {
    validate_layer_edit(layers, library, locked, layer_id, &value)?;
    if value.source == NprGeometrySource::Paper {
        return Err("Paper has no tool appearance".into());
    }
    let old = value.brush.as_ref().map(|instance| instance.brush.clone());
    let mut brush = old
        .as_ref()
        .map(|reference| library.resolve(reference).cloned())
        .transpose()?
        .unwrap_or_default();
    if name.trim().is_empty() || name.len() > 80 {
        return Err("appearance name must contain 1..=80 characters".into());
    }
    if brush.name != name {
        brush.id = (1..)
            .map(|index| format!("appearance-{index}"))
            .find(|id| !library.brushes.contains_key(id))
            .expect("unbounded id sequence");
    }
    brush.name = name.trim().into();
    brush.version = library
        .brushes
        .get(&brush.id)
        .and_then(|versions| versions.iter().map(|b| b.version).max())
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("appearance revision limit reached")?;
    brush.tool = value.tool.or(brush.tool);
    brush.paint = value.paint.or(brush.paint);
    if let Some(instance) = &value.brush {
        macro_rules! capture { ($($field:ident),*) => { $(if let Some(v) = instance.$field { brush.$field = v; })* }; }
        capture!(
            width,
            taper,
            softness,
            spacing,
            pressure_profile,
            irregularity,
            correction,
            dryness,
            seed
        );
    }
    let reference = BrushReference {
        id: brush.id.clone(),
        version: brush.version,
    };
    let mut edited = layers.clone();
    *edited.layer_mut(layer_id).expect("validated") = value;
    for layer in &mut edited.layers {
        let matching = update_matching
            && old.is_some()
            && layer.brush.as_ref().map(|b| &b.brush) == old.as_ref();
        if layer.id != layer_id && !matching {
            continue;
        }
        if locked.contains(&layer.id) {
            return Err(format!("layer is locked: {}", layer.id));
        }
        if layer.id == layer_id {
            layer.brush = Some(BrushInstance {
                brush: reference.clone(),
                ..Default::default()
            });
        } else if let Some(instance) = &mut layer.brush {
            instance.brush = reference.clone();
        }
    }
    let mut updated_library = library.clone();
    updated_library.add_version(brush)?;
    edited.validate()?;
    edited.validate_brushes(&updated_library)?;
    *layers = edited;
    *library = updated_library;
    Ok(())
}
