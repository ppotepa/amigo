//! Prepared, backend-independent drawing surfaces.
//!
//! A render packet may be rebuilt for a moving camera every frame, while mesh
//! adjacency is a property of one source revision.  Keeping these concerns
//! together prevents each domain plugin from inventing a slightly different
//! geometry/topology cache.

use crate::{
    NprGeometry, NprSubdivisionError, TopologyEdge, build_topology, subdivide_smooth_proxy,
};
use std::collections::BTreeMap;

/// Stable content identifier for a prepared drawing surface.
///
/// This is not a replacement for an asset-system revision.  Asset owners can
/// key their cache by their own revision and use this identifier to detect an
/// accidental geometry change within that revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NprSurfaceContentId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub struct NprPreparedSurface {
    geometry: NprGeometry,
    topology: Vec<TopologyEdge>,
    content_id: NprSurfaceContentId,
}

/// Fixed, revision-scoped policy for a smooth drawing proxy. It is deliberately
/// independent of viewport and camera state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprSmoothProxyPolicy {
    pub levels: u8,
    pub crease_angle: f32,
    pub max_triangles: usize,
}

impl Default for NprSmoothProxyPolicy {
    fn default() -> Self {
        Self {
            levels: 1,
            crease_angle: 1.2,
            max_triangles: 250_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprPreparedSurfaceVariants {
    source: NprPreparedSurface,
    smooth_proxies: BTreeMap<SmoothProxyKey, NprPreparedSurface>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmoothProxyKey {
    levels: u8,
    crease_angle_bits: u32,
    max_triangles: usize,
}

impl NprPreparedSurfaceVariants {
    pub fn new(geometry: NprGeometry) -> Self {
        Self {
            source: NprPreparedSurface::new(geometry),
            smooth_proxies: BTreeMap::new(),
        }
    }

    pub fn source(&self) -> &NprPreparedSurface {
        &self.source
    }

    pub fn smooth_proxy(
        &mut self,
        policy: NprSmoothProxyPolicy,
    ) -> Result<&NprPreparedSurface, NprSubdivisionError> {
        if policy.levels == 0 {
            return Ok(&self.source);
        }
        let key = SmoothProxyKey {
            levels: policy.levels,
            crease_angle_bits: canonical_angle_bits(policy.crease_angle),
            max_triangles: policy.max_triangles,
        };
        if !self.smooth_proxies.contains_key(&key) {
            // A slider may visit many intermediate crease values. Bound only
            // cache residency; the requested policy is still prepared exactly.
            if self.smooth_proxies.len() >= 8 {
                self.smooth_proxies.clear();
            }
            let proxy = subdivide_smooth_proxy(
                self.source.geometry(),
                policy.levels,
                f32::from_bits(key.crease_angle_bits),
                policy.max_triangles,
            )?;
            self.smooth_proxies
                .insert(key, NprPreparedSurface::new(proxy));
        }
        Ok(self
            .smooth_proxies
            .get(&key)
            .expect("newly inserted NPR smooth proxy must be present"))
    }
}

fn canonical_angle_bits(value: f32) -> u32 {
    let value = if value.is_finite() {
        value.clamp(0.0, std::f32::consts::PI)
    } else {
        NprSmoothProxyPolicy::default().crease_angle
    };
    if value == 0.0 { 0 } else { value.to_bits() }
}

impl NprPreparedSurface {
    pub fn new(geometry: NprGeometry) -> Self {
        let content_id = NprSurfaceContentId(hash_geometry(&geometry));
        let topology = build_topology(&geometry);
        Self {
            geometry,
            topology,
            content_id,
        }
    }

    pub fn from_indexed(positions: &[[f32; 3]], indices: &[u32]) -> Result<Self, String> {
        NprGeometry::from_indexed(positions, indices).map(Self::new)
    }

    pub fn geometry(&self) -> &NprGeometry {
        &self.geometry
    }

    pub fn topology(&self) -> &[TopologyEdge] {
        &self.topology
    }

    pub fn content_id(&self) -> NprSurfaceContentId {
        self.content_id
    }
}

fn hash_geometry(geometry: &NprGeometry) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    fn byte(hash: &mut u64, value: u8) {
        *hash = (*hash ^ u64::from(value)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn bytes(hash: &mut u64, values: &[u8]) {
        for value in values {
            byte(hash, *value);
        }
    }
    bytes(&mut hash, &(geometry.vertices.len() as u64).to_le_bytes());
    for vertex in &geometry.vertices {
        for value in vertex.position.to_array() {
            // Signed zero does not change a drawing surface.
            let bits = if value == 0.0 { 0 } else { value.to_bits() };
            bytes(&mut hash, &bits.to_le_bytes());
        }
    }
    bytes(&mut hash, &(geometry.triangles.len() as u64).to_le_bytes());
    for triangle in &geometry.triangles {
        for index in triangle {
            bytes(&mut hash, &index.to_le_bytes());
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ComicInk, NprDebugView, PerspectiveCamera, build_packet_for_surface,
        build_packet_with_topology,
    };
    use glam::Vec3;

    #[test]
    fn prepared_surface_reuses_topology_and_has_stable_content_id() {
        let surface = NprPreparedSurface::new(NprGeometry::canonical_cube());
        assert_eq!(surface.topology().len(), 18);
        assert_eq!(
            surface.content_id(),
            NprPreparedSurface::new(NprGeometry::canonical_cube()).content_id()
        );
    }

    #[test]
    fn content_id_changes_with_surface_geometry() {
        let cube = NprPreparedSurface::new(NprGeometry::canonical_cube());
        let wedge = NprPreparedSurface::new(NprGeometry::wedge());
        assert_ne!(cube.content_id(), wedge.content_id());
    }

    #[test]
    fn prepared_surface_packet_matches_explicit_topology_path() {
        let surface = NprPreparedSurface::new(NprGeometry::canonical_cube());
        let camera = PerspectiveCamera {
            position: Vec3::new(3.0, 2.0, 4.0),
            forward: Vec3::new(-3.0, -2.0, -4.0).normalize(),
            up: Vec3::Y,
            vertical_fov: 0.9,
            near: 0.05,
            aspect: 1.0,
        };
        assert_eq!(
            build_packet_for_surface(
                &surface,
                camera,
                [512, 512],
                ComicInk::default(),
                42,
                NprDebugView::Final,
            ),
            build_packet_with_topology(
                surface.geometry(),
                surface.topology(),
                camera,
                [512, 512],
                ComicInk::default(),
                42,
                NprDebugView::Final,
            )
        );
    }

    #[test]
    fn variants_cache_proxy_by_surface_policy() {
        let mut variants = NprPreparedSurfaceVariants::new(NprGeometry::icosphere());
        let source_id = variants.source().content_id();
        let first_id = variants
            .smooth_proxy(NprSmoothProxyPolicy::default())
            .unwrap()
            .content_id();
        let second_id = variants
            .smooth_proxy(NprSmoothProxyPolicy::default())
            .unwrap()
            .content_id();
        assert_ne!(source_id, first_id);
        assert_eq!(first_id, second_id);
    }
}
