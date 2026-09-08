//! Prepared, backend-independent drawing surfaces.
//!
//! A render packet may be rebuilt for a moving camera every frame, while mesh
//! adjacency is a property of one source revision.  Keeping these concerns
//! together prevents each domain plugin from inventing a slightly different
//! geometry/topology cache.

use crate::{NprGeometry, TopologyEdge, build_topology};

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
}
