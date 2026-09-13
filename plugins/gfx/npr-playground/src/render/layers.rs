//! Per-entry extraction over one shared, prepared model surface.
use amigo_render_npr::*;
use std::{collections::BTreeMap, time::Instant};

pub(super) fn build_drawing(
    surface: &NprPreparedSurface,
    source_surface: &NprPreparedSurface,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug: NprDebugView,
    layers: &NprStyleLayers,
    library: &BrushLibrary,
    marks: &[NprConstructionMark],
) -> Result<(NprRenderPacket, BTreeMap<String, NprLayerDiagnostics>), String> {
    layers.validate_brushes(library)?;
    let mut cache: Vec<(ComicInk, u64, NprRenderPacket)> = Vec::new();
    let mut output: Option<NprRenderPacket> = None;
    let mut diagnostics = BTreeMap::new();
    let mut entries = layers
        .layers
        .iter()
        .filter(|layer| layer.enabled && layer.source != NprGeometrySource::Paper)
        .collect::<Vec<_>>();
    // Extraction priority bounds transient memory; composition still follows
    // the authored stack, not this traversal order.
    entries.sort_by_key(|layer| match layer.source {
        NprGeometrySource::Silhouette
        | NprGeometrySource::Creases
        | NprGeometrySource::Construction => 0,
        NprGeometrySource::FormLines => 1,
        NprGeometrySource::ShadowHatch => 2,
        _ => 3,
    });
    let mut used_bytes = 0;
    let mut budget_rejected = 0;
    for layer in entries {
        let effective = layer.extraction_style(style, library)?;
        let appearance_seed = layer
            .brush
            .as_ref()
            .map(|instance| {
                instance
                    .seed
                    .or_else(|| {
                        library
                            .resolve(&instance.brush)
                            .ok()
                            .map(|brush| brush.seed)
                    })
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let entry_seed = seed ^ appearance_seed;
        let started = Instant::now();
        let source = if let Some((_, _, packet)) = cache
            .iter()
            .find(|(key, key_seed, _)| *key == effective && *key_seed == entry_seed)
        {
            packet.clone()
        } else {
            let mut packet =
                build_packet_for_surface(surface, camera, viewport, effective, entry_seed, debug);
            append_construction_marks(
                &mut packet,
                source_surface,
                camera,
                viewport,
                effective,
                entry_seed,
                marks,
            )
            .map_err(|error| error.to_string())?;
            // Bound transient source storage independently of the 32-entry document limit.
            if cache.len() == 2 {
                cache.remove(0);
            }
            cache.push((effective, entry_seed, packet.clone()));
            packet
        };
        let extraction_micros = started.elapsed().as_micros() as u64;
        let mut contribution = source.clone();
        let started = Instant::now();
        let mut material_layer = layer.clone();
        // Density/spacing have already driven this entry's generator.
        material_layer.hatch = None;
        let material = NprStyleLayers {
            layers: vec![material_layer],
        };
        material.apply_tools_with_library(&mut contribution, effective, Some(library))?;
        contribution.strokes.retain(|stroke| {
            let bytes = super::stroke_data_bytes(stroke);
            if used_bytes + bytes > super::MAX_DRAWING_STROKE_DATA_BYTES {
                budget_rejected += 1;
                false
            } else {
                used_bytes += bytes;
                true
            }
        });
        contribution
            .fills
            .retain(|triangle| triangle.layer_id.as_deref() == Some(&layer.id));
        contribution
            .underpainting
            .retain(|triangle| triangle.layer_id.as_deref() == Some(&layer.id));
        diagnostics.extend(
            NprStyleLayers {
                layers: vec![layer.clone()],
            }
            .diagnostics(
                &source,
                &contribution,
                extraction_micros,
                started.elapsed().as_micros() as u64,
            ),
        );
        let drawing = output.get_or_insert_with(|| {
            let mut packet = source.clone();
            packet.strokes.clear();
            packet.fills.clear();
            packet.underpainting.clear();
            packet
        });
        drawing.strokes.extend(contribution.strokes);
        drawing.fills.extend(contribution.fills);
        drawing.underpainting.extend(contribution.underpainting);
    }
    let mut packet = output.unwrap_or_else(|| {
        let mut packet = build_packet_for_surface(surface, camera, viewport, style, seed, debug);
        packet.strokes.clear();
        packet.fills.clear();
        packet.underpainting.clear();
        packet
    });
    for (id, diagnostic) in layers.diagnostics(&packet, &packet, 0, 0) {
        diagnostics.entry(id).or_insert(diagnostic);
    }
    packet.stats.strokes = packet.strokes.len();
    packet.stats.hatching_strokes = packet
        .strokes
        .iter()
        .filter(|s| s.role == StrokeRole::Tone)
        .count();
    packet.stats.form_line_strokes = packet
        .strokes
        .iter()
        .filter(|s| s.role == StrokeRole::FormLine)
        .count();
    packet.stats.construction_marks = packet
        .strokes
        .iter()
        .filter(|s| s.role == StrokeRole::Construction)
        .count();
    packet.stats.stroke_vertices = packet.strokes.iter().map(|s| s.vertices.len()).sum();
    packet.stats.stroke_indices = packet.strokes.iter().map(|s| s.indices.len()).sum();
    packet.stats.underpainting_triangles = packet.underpainting.len();
    packet.stats.stroke_data_bytes = used_bytes;
    packet.stats.stroke_budget_rejected += budget_rejected;
    packet.stats.stroke_budget_exhausted |= budget_rejected > 0;
    Ok((packet, diagnostics))
}
