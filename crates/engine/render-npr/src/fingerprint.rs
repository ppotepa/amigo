//! Stable, backend-independent evidence for an NPR render packet.
//!
//! A pixel image is useful for an art review, but is a poor primary contract for
//! a renderer: driver rasterisation, colour conversion, and anti-aliasing can
//! obscure whether geometry or stroke planning changed.  This fingerprint locks
//! the packet that the backend receives.  It is deliberately versioned so a
//! future intentional contract change is explicit in a review.

use crate::{
    FeatureClass, NprDebugView, NprFillTriangle, NprRenderPacket, NprRenderStats, StrokeRole,
    StrokeVertex, TessellatedStroke,
};

/// Increment when the byte order or field set hashed by [`NprPacketFingerprint`]
/// changes.  Do not use this as a rendering version; it is a test-artifact format.
pub const NPR_PACKET_FINGERPRINT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NprPacketFingerprint {
    pub version: u16,
    pub hash: u64,
    pub occluders: usize,
    pub fills: usize,
    pub strokes: usize,
    pub stroke_vertices: usize,
    pub stroke_indices: usize,
}

impl NprPacketFingerprint {
    pub fn from_packet(packet: &NprRenderPacket) -> Self {
        let mut hasher = PacketHasher::default();
        hasher.u16(NPR_PACKET_FINGERPRINT_VERSION);
        hash_triangles(&mut hasher, &packet.occluders);
        hash_triangles(&mut hasher, &packet.fills);
        hasher.vec4(packet.background.to_array());
        hasher.u8(debug_view_tag(packet.debug_view));
        hasher.vec4(packet.ink.to_array());
        hash_strokes(&mut hasher, &packet.strokes);
        hash_stats(&mut hasher, &packet.stats);
        Self {
            version: NPR_PACKET_FINGERPRINT_VERSION,
            hash: hasher.finish(),
            occluders: packet.occluders.len(),
            fills: packet.fills.len(),
            strokes: packet.strokes.len(),
            stroke_vertices: packet
                .strokes
                .iter()
                .map(|stroke| stroke.vertices.len())
                .sum(),
            stroke_indices: packet
                .strokes
                .iter()
                .map(|stroke| stroke.indices.len())
                .sum(),
        }
    }
}

impl NprRenderPacket {
    /// Returns deterministic CPU-side evidence of the complete backend packet.
    pub fn fingerprint(&self) -> NprPacketFingerprint {
        NprPacketFingerprint::from_packet(self)
    }
}

#[derive(Debug)]
struct PacketHasher(u64);

impl Default for PacketHasher {
    fn default() -> Self {
        // FNV-1a is specified here rather than `DefaultHasher`, whose algorithm
        // is intentionally not a stable public contract.
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl PacketHasher {
    fn finish(self) -> u64 {
        self.0
    }
    fn u8(&mut self, value: u8) {
        self.0 = (self.0 ^ u64::from(value)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn u16(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }
    fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }
    fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }
    fn usize(&mut self, value: usize) {
        self.u64(value as u64);
    }
    fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }
    fn f32(&mut self, value: f32) {
        // Packets must not carry NaN, but canonicalising it keeps a diagnostic
        // fingerprint informative should validation fail upstream.
        let bits = if value.is_nan() {
            f32::NAN.to_bits()
        } else if value == 0.0 {
            0
        } else {
            value.to_bits()
        };
        self.u32(bits);
    }
    fn vec2(&mut self, value: [f32; 2]) {
        value.into_iter().for_each(|component| self.f32(component));
    }
    fn vec4(&mut self, value: [f32; 4]) {
        value.into_iter().for_each(|component| self.f32(component));
    }
    fn bytes(&mut self, bytes: &[u8]) {
        bytes.iter().copied().for_each(|byte| self.u8(byte));
    }
}

fn hash_triangles(hasher: &mut PacketHasher, triangles: &[NprFillTriangle]) {
    hasher.usize(triangles.len());
    for triangle in triangles {
        for position in triangle.positions {
            hasher.vec2(position.to_array());
        }
        hasher.vec4(triangle.color.to_array());
        triangle
            .depths
            .into_iter()
            .for_each(|depth| hasher.f32(depth));
    }
}

fn hash_strokes(hasher: &mut PacketHasher, strokes: &[TessellatedStroke]) {
    hasher.usize(strokes.len());
    for stroke in strokes {
        hasher.u32(stroke.id);
        hasher.u8(feature_class_tag(stroke.class));
        hasher.u8(stroke_role_tag(stroke.role));
        hasher.bool(stroke.correction);
        hasher.usize(stroke.vertices.len());
        for vertex in &stroke.vertices {
            hash_vertex(hasher, *vertex);
        }
        hasher.usize(stroke.indices.len());
        stroke
            .indices
            .iter()
            .copied()
            .for_each(|index| hasher.u32(index));
    }
}

fn hash_vertex(hasher: &mut PacketHasher, vertex: StrokeVertex) {
    hasher.vec2(vertex.position.to_array());
    hasher.f32(vertex.width);
    hasher.u32(vertex.id);
    hasher.f32(vertex.depth);
    hasher.f32(vertex.pressure);
    hasher.f32(vertex.coverage);
    hasher.f32(vertex.grain);
    hasher.f32(vertex.edge);
    hasher.f32(vertex.edge_softness);
    hasher.f32(vertex.paper_tooth);
    hasher.f32(vertex.dryness);
}

fn hash_stats(hasher: &mut PacketHasher, stats: &NprRenderStats) {
    hasher.usize(stats.geometry);
    hasher.usize(stats.surface_source_vertices);
    hasher.usize(stats.surface_proxy_vertices);
    hasher.usize(stats.surface_source_triangles);
    hasher.usize(stats.surface_proxy_triangles);
    hasher.usize(stats.topology_edges);
    hasher.usize(stats.feature_segments);
    hasher.usize(stats.feature_candidates);
    hasher.usize(stats.feature_rejected);
    hasher.usize(stats.smooth_contour_spans);
    hasher.usize(stats.suggestive_contour_spans);
    hasher.usize(stats.silhouettes);
    hasher.usize(stats.creases);
    hasher.usize(stats.strokes);
    hasher.usize(stats.stroke_vertices);
    hasher.usize(stats.stroke_indices);
    hasher.usize(stats.hatching_strokes);
    hasher.usize(stats.hatching_correction_strokes);
    hasher.f32(stats.graphite_mass);
    hasher.usize(stats.hatching_candidates);
    hasher.usize(stats.hatching_rejected);
    hasher.usize(stats.hatching_confidence_rejected);
    hasher.usize(stats.construction_marks);
    hasher.usize(stats.construction_rejected);
    hasher.u8(stats.hatching_lod_tier);
    hasher.bool(stats.hatching_budget_exhausted);
    hasher.usize(stats.stroke_budget_rejected);
    hasher.bool(stats.stroke_budget_exhausted);
    hasher.usize(stats.temporal_retained_strokes);
    hasher.usize(stats.temporal_entering_strokes);
    hasher.u32(stats.gesture_variant_epoch);
    hasher.usize(stats.stroke_data_bytes);
    stats
        .viewport
        .into_iter()
        .for_each(|value| hasher.u32(value));
}

fn feature_class_tag(value: FeatureClass) -> u8 {
    match value {
        FeatureClass::Boundary => 0,
        FeatureClass::Silhouette => 1,
        FeatureClass::Crease => 2,
    }
}
fn stroke_role_tag(value: StrokeRole) -> u8 {
    match value {
        StrokeRole::Feature => 0,
        StrokeRole::Tone => 1,
        StrokeRole::Construction => 2,
    }
}
fn debug_view_tag(value: NprDebugView) -> u8 {
    match value {
        NprDebugView::Final => 0,
        NprDebugView::FeatureClasses => 1,
        NprDebugView::StrokeIds => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec4};

    #[test]
    fn fingerprint_is_stable_and_changes_with_backend_packet_data() {
        let mut packet = NprRenderPacket {
            occluders: vec![NprFillTriangle {
                positions: [Vec2::ZERO, Vec2::X, Vec2::Y],
                color: Vec4::ONE,
                depths: [0.1, 0.2, 0.3],
            }],
            fills: Vec::new(),
            strokes: Vec::new(),
            background: Vec4::ZERO,
            debug_view: NprDebugView::Final,
            ink: Vec4::ONE,
            stats: NprRenderStats::default(),
        };
        let first = packet.fingerprint();
        assert_eq!(first, packet.fingerprint());
        packet.occluders[0].depths[0] = 0.4;
        assert_ne!(first.hash, packet.fingerprint().hash);
    }
}
