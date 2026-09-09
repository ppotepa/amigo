//! Prepared, backend-independent drawing surfaces.
//!
//! A render packet may be rebuilt for a moving camera every frame, while mesh
//! adjacency is a property of one source revision.  Keeping these concerns
//! together prevents each domain plugin from inventing a slightly different
//! geometry/topology cache.

use crate::{
    build_topology, subdivide_smooth_proxy_with_provenance, NprGeometry, NprSmoothProxyGeometry,
    NprSourceTriangleMapping, NprSubdivisionError, TopologyEdge,
};
use glam::Vec3;
use std::collections::BTreeMap;
use std::fmt;

/// Stable content identifier for a prepared drawing surface.
///
/// This is not a replacement for an asset-system revision.  Asset owners can
/// key their cache by their own revision and use this identifier to detect an
/// accidental geometry change within that revision.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct NprSurfaceContentId(pub u64);

/// A local-space point on one immutable source-surface triangle.
///
/// The content identifier makes an anchor revision-scoped: clients must not
/// accidentally reuse a stroke point after an asset changed.  The anchor is
/// intentionally expressed against the source surface, never a view-dependent
/// smooth proxy, so it remains meaningful while camera and proxy policy vary.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NprSurfaceAnchor {
    pub content_id: NprSurfaceContentId,
    pub triangle: u32,
    pub barycentric: [f32; 3],
}

/// Local-space result of resolving a [`NprSurfaceAnchor`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprSurfaceSample {
    pub position: Vec3,
    pub normal: Vec3,
}

/// Nearest local-space intersection between a ray and a prepared surface.
///
/// The anchor always refers to the source revision. This lets an authoring
/// tool use a smooth proxy for visual picking without persisting a point in a
/// transient proxy mesh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprSurfaceRayHit {
    pub anchor: NprSurfaceAnchor,
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprSurfaceAnchorError {
    ContentMismatch,
    TriangleOutOfRange,
    InvalidBarycentric,
}

impl fmt::Display for NprSurfaceAnchorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ContentMismatch => "surface anchor belongs to a different content revision",
            Self::TriangleOutOfRange => "surface anchor triangle is out of range",
            Self::InvalidBarycentric => "surface anchor barycentric coordinates are invalid",
        })
    }
}

impl std::error::Error for NprSurfaceAnchorError {}

#[derive(Debug, Clone, PartialEq)]
pub struct NprPreparedSurface {
    geometry: NprGeometry,
    topology: Vec<TopologyEdge>,
    content_id: NprSurfaceContentId,
    source_content_id: NprSurfaceContentId,
    source_triangles: Option<Vec<NprSourceTriangleMapping>>,
}

/// Fixed, revision-scoped policy for a smooth drawing proxy. It is deliberately
/// independent of viewport and camera state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprSmoothProxyPolicy {
    pub levels: u8,
    pub crease_angle: f32,
    /// Relative to the source bounding-box diagonal. Zero welds only exactly
    /// coincident positions; a positive value absorbs importer seam jitter.
    pub weld_relative_tolerance: f32,
    pub max_triangles: usize,
}

impl Default for NprSmoothProxyPolicy {
    fn default() -> Self {
        Self {
            levels: 1,
            crease_angle: 1.2,
            weld_relative_tolerance: 1.0e-5,
            max_triangles: 250_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprPreparedSurfaceVariants {
    source: NprPreparedSurface,
    smooth_drawing_sources: BTreeMap<u32, NprGeometry>,
    smooth_proxies: BTreeMap<SmoothProxyKey, NprPreparedSurface>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmoothProxyKey {
    levels: u8,
    crease_angle_bits: u32,
    weld_relative_tolerance_bits: u32,
    max_triangles: usize,
}

impl NprPreparedSurfaceVariants {
    pub fn new(geometry: NprGeometry) -> Self {
        Self {
            source: NprPreparedSurface::new(geometry),
            smooth_drawing_sources: BTreeMap::new(),
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
        // UV/normal seams often duplicate vertex indices without splitting the
        // physical surface. Weld only in the explicitly authored Smooth path:
        // Polygonal assets retain their literal topology, while smooth drawing
        // contours stop promoting importer seams to boundaries.
        let weld_relative_tolerance_bits = canonical_weld_tolerance_bits(policy.weld_relative_tolerance);
        let source_vertex_count = self.source.geometry().vertices.len();
        if policy.levels == 0
            && self
                .smooth_drawing_source(weld_relative_tolerance_bits)
                .vertices
                .len()
                == source_vertex_count
        {
            return Ok(&self.source);
        }
        let key = SmoothProxyKey {
            levels: policy.levels,
            crease_angle_bits: canonical_angle_bits(policy.crease_angle),
            weld_relative_tolerance_bits,
            max_triangles: policy.max_triangles,
        };
        if !self.smooth_proxies.contains_key(&key) {
            // A slider may visit many intermediate crease values. Bound only
            // cache residency; the requested policy is still prepared exactly.
            if self.smooth_proxies.len() >= 8 {
                self.smooth_proxies.clear();
            }
            let proxy = {
                let drawing_source = self.smooth_drawing_source(weld_relative_tolerance_bits);
                subdivide_smooth_proxy_with_provenance(
                    drawing_source,
                    policy.levels,
                    f32::from_bits(key.crease_angle_bits),
                    policy.max_triangles,
                )?
            };
            self.smooth_proxies.insert(
                key,
                NprPreparedSurface::from_proxy(proxy, self.source.content_id),
            );
        }
        Ok(self
            .smooth_proxies
            .get(&key)
            .expect("newly inserted NPR smooth proxy must be present"))
    }

    fn smooth_drawing_source(&mut self, tolerance_bits: u32) -> &NprGeometry {
        self.smooth_drawing_sources
            .entry(tolerance_bits)
            .or_insert_with(|| {
                self.source
                    .geometry()
                    .welded_nearby_vertices(f32::from_bits(tolerance_bits))
            })
    }
}

fn canonical_angle_bits(value: f32) -> u32 {
    let value = if value.is_finite() {
        value.clamp(0.0, std::f32::consts::PI)
    } else {
        NprSmoothProxyPolicy::default().crease_angle
    };
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

fn canonical_weld_tolerance_bits(value: f32) -> u32 {
    let value = if value.is_finite() {
        value.clamp(0.0, 1.0e-2)
    } else {
        NprSmoothProxyPolicy::default().weld_relative_tolerance
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
            source_content_id: content_id,
            source_triangles: None,
        }
    }

    fn from_proxy(proxy: NprSmoothProxyGeometry, source_content_id: NprSurfaceContentId) -> Self {
        let content_id = NprSurfaceContentId(hash_geometry(&proxy.geometry));
        let topology = build_topology(&proxy.geometry);
        Self {
            geometry: proxy.geometry,
            topology,
            content_id,
            source_content_id,
            source_triangles: Some(proxy.source_triangles),
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

    /// Content revision of the mesh on which persistent marks are authored.
    pub fn source_content_id(&self) -> NprSurfaceContentId {
        self.source_content_id
    }

    /// Creates a validated anchor in this immutable surface revision.
    pub fn anchor(
        &self,
        triangle: u32,
        barycentric: [f32; 3],
    ) -> Result<NprSurfaceAnchor, NprSurfaceAnchorError> {
        if triangle as usize >= self.geometry.triangles.len() {
            return Err(NprSurfaceAnchorError::TriangleOutOfRange);
        }
        if !valid_barycentric(barycentric) {
            return Err(NprSurfaceAnchorError::InvalidBarycentric);
        }
        Ok(NprSurfaceAnchor {
            content_id: self.content_id,
            triangle,
            barycentric,
        })
    }

    /// Resolves a source-local anchor without applying an object transform.
    ///
    /// Callers apply their current object transform after sampling.  That keeps
    /// authored stroke placement stable under object animation and avoids
    /// baking camera/proxy state into a drawing mark.
    pub fn sample(
        &self,
        anchor: NprSurfaceAnchor,
    ) -> Result<NprSurfaceSample, NprSurfaceAnchorError> {
        if anchor.content_id != self.content_id {
            return Err(NprSurfaceAnchorError::ContentMismatch);
        }
        if !valid_barycentric(anchor.barycentric) {
            return Err(NprSurfaceAnchorError::InvalidBarycentric);
        }
        let triangle = self
            .geometry
            .triangles
            .get(anchor.triangle as usize)
            .ok_or(NprSurfaceAnchorError::TriangleOutOfRange)?;
        let [a, b, c] = triangle.map(|index| self.geometry.vertices[index as usize].position);
        let [wa, wb, wc] = anchor.barycentric;
        let normal = (b - a).cross(c - a).normalize_or_zero();
        Ok(NprSurfaceSample {
            position: a * wa + b * wb + c * wc,
            normal,
        })
    }

    /// Converts a proxy-chart coordinate into an anchor on the source mesh.
    ///
    /// On an unmodified surface this is a direct face-local anchor. On a
    /// smooth proxy, the fixed subdivision chart determines the equivalent
    /// source triangle and barycentric coordinates.
    pub fn source_anchor(
        &self,
        triangle: u32,
        barycentric: [f32; 3],
    ) -> Result<NprSurfaceAnchor, NprSurfaceAnchorError> {
        if triangle as usize >= self.geometry.triangles.len() {
            return Err(NprSurfaceAnchorError::TriangleOutOfRange);
        }
        if !valid_barycentric(barycentric) {
            return Err(NprSurfaceAnchorError::InvalidBarycentric);
        }
        let (source_triangle, barycentric) = self
            .source_triangles
            .as_ref()
            .and_then(|mappings| mappings.get(triangle as usize))
            .map(|mapping| {
                let [a, b, c] = mapping.corners;
                let [wa, wb, wc] = barycentric;
                (
                    mapping.source_triangle,
                    [
                        a[0] * wa + b[0] * wb + c[0] * wc,
                        a[1] * wa + b[1] * wb + c[1] * wc,
                        a[2] * wa + b[2] * wb + c[2] * wc,
                    ],
                )
            })
            .unwrap_or((triangle, barycentric));
        Ok(NprSurfaceAnchor {
            content_id: self.source_content_id,
            triangle: source_triangle,
            barycentric,
        })
    }

    /// Converts a point on a prepared triangle into its source-surface anchor.
    pub fn source_anchor_at_point(
        &self,
        triangle: u32,
        point: Vec3,
    ) -> Result<NprSurfaceAnchor, NprSurfaceAnchorError> {
        let triangle_vertices = self
            .geometry
            .triangles
            .get(triangle as usize)
            .ok_or(NprSurfaceAnchorError::TriangleOutOfRange)?
            .map(|index| self.geometry.vertices[index as usize].position);
        let barycentric = barycentric_at_point(triangle_vertices, point)
            .ok_or(NprSurfaceAnchorError::InvalidBarycentric)?;
        self.source_anchor(triangle, barycentric)
    }

    /// Finds the nearest front or back-facing triangle hit in local space.
    ///
    /// `origin` and `direction` must already be transformed from viewport
    /// space into this surface's local space. The method intentionally does
    /// not apply an object transform or camera policy, so it can be shared by
    /// an in-game authoring panel and a future editor.
    pub fn raycast(&self, origin: Vec3, direction: Vec3) -> Option<NprSurfaceRayHit> {
        const EPSILON: f32 = 1e-6;
        if !origin.is_finite() || !direction.is_finite() || direction.length_squared() <= EPSILON {
            return None;
        }

        self.geometry
            .triangles
            .iter()
            .enumerate()
            .filter_map(|(triangle_index, triangle)| {
                let vertices = triangle.map(|index| {
                    self.geometry
                        .vertices
                        .get(index as usize)
                        .map(|vertex| vertex.position)
                });
                let [Some(a), Some(b), Some(c)] = vertices else {
                    return None;
                };
                let (distance, barycentric) =
                    ray_triangle_intersection(origin, direction, [a, b, c])?;
                (distance > EPSILON).then_some((
                    triangle_index as u32,
                    [a, b, c],
                    distance,
                    barycentric,
                ))
            })
            .min_by(|left, right| left.2.total_cmp(&right.2))
            .and_then(|(triangle, vertices, distance, barycentric)| {
                let anchor = self.source_anchor(triangle, barycentric).ok()?;
                Some(NprSurfaceRayHit {
                    anchor,
                    position: origin + direction * distance,
                    normal: (vertices[1] - vertices[0])
                        .cross(vertices[2] - vertices[0])
                        .normalize_or_zero(),
                    distance,
                })
            })
    }
}

fn valid_barycentric(value: [f32; 3]) -> bool {
    const EPSILON: f32 = 1e-4;
    value
        .iter()
        .all(|weight| weight.is_finite() && *weight >= -EPSILON)
        && (value.iter().sum::<f32>() - 1.0).abs() <= EPSILON
}

fn barycentric_at_point(triangle: [Vec3; 3], point: Vec3) -> Option<[f32; 3]> {
    let [a, b, c] = triangle;
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let dot_ab_ab = ab.dot(ab);
    let dot_ab_ac = ab.dot(ac);
    let dot_ac_ac = ac.dot(ac);
    let denominator = dot_ab_ab * dot_ac_ac - dot_ab_ac * dot_ab_ac;
    (denominator.abs() > 1e-10).then(|| {
        let b = (dot_ac_ac * ap.dot(ab) - dot_ab_ac * ap.dot(ac)) / denominator;
        let c = (dot_ab_ab * ap.dot(ac) - dot_ab_ac * ap.dot(ab)) / denominator;
        [1.0 - b - c, b, c]
    })
}

fn ray_triangle_intersection(
    origin: Vec3,
    direction: Vec3,
    [a, b, c]: [Vec3; 3],
) -> Option<(f32, [f32; 3])> {
    const EPSILON: f32 = 1e-6;
    let ab = b - a;
    let ac = c - a;
    let perpendicular = direction.cross(ac);
    let determinant = ab.dot(perpendicular);
    if determinant.abs() <= EPSILON {
        return None;
    }
    let inverse_determinant = determinant.recip();
    let offset = origin - a;
    let b_weight = offset.dot(perpendicular) * inverse_determinant;
    if !(0.0..=1.0).contains(&b_weight) {
        return None;
    }
    let q = offset.cross(ab);
    let c_weight = direction.dot(q) * inverse_determinant;
    if c_weight < 0.0 || b_weight + c_weight > 1.0 {
        return None;
    }
    let distance = ac.dot(q) * inverse_determinant;
    (distance.is_finite()).then_some((distance, [1.0 - b_weight - c_weight, b_weight, c_weight]))
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
        build_packet_for_surface, build_packet_with_topology, ComicInk, NprDebugView,
        PerspectiveCamera,
    };
    use glam::{Mat4, Vec3};

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

    #[test]
    fn smooth_proxy_welds_split_import_indices_without_subdivision() {
        let split = NprGeometry::from_indexed(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            &[0, 1, 2, 3, 4, 5],
        )
        .unwrap();
        let mut variants = NprPreparedSurfaceVariants::new(split);
        let source = variants.source().clone();
        let proxy = variants
            .smooth_proxy(NprSmoothProxyPolicy {
                levels: 0,
                ..NprSmoothProxyPolicy::default()
            })
            .unwrap();

        assert_eq!(proxy.geometry().vertices.len(), 4);
        assert_eq!(proxy.source_content_id(), source.content_id());
        assert!(proxy
            .topology()
            .iter()
            .any(|edge| edge.faces[0] != u32::MAX && edge.faces[1] != u32::MAX));
        let anchor = proxy.source_anchor(0, [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(anchor.content_id, source.content_id());
        assert_eq!(anchor.triangle, 0);
    }

    #[test]
    fn smooth_proxy_welds_small_importer_position_jitter() {
        let split = NprGeometry::from_indexed(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0 + 0.000_001, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0 + 0.000_001, 0.0],
            ],
            &[0, 1, 2, 3, 4, 5],
        )
        .unwrap();
        let mut variants = NprPreparedSurfaceVariants::new(split);
        let proxy = variants
            .smooth_proxy(NprSmoothProxyPolicy {
                levels: 0,
                ..NprSmoothProxyPolicy::default()
            })
            .unwrap();

        assert_eq!(proxy.geometry().vertices.len(), 4);
        assert!(proxy
            .geometry()
            .triangles
            .iter()
            .all(|triangle| triangle[0] != triangle[1] && triangle[1] != triangle[2]));
    }

    #[test]
    fn smooth_proxy_weld_tolerance_is_an_explicit_surface_policy() {
        let split = NprGeometry::from_indexed(
            &[
                [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
                [1.0 + 0.000_001, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0 + 0.000_001, 0.0],
            ],
            &[0, 1, 2, 3, 4, 5],
        ).unwrap();
        let mut variants = NprPreparedSurfaceVariants::new(split);
        let exact = variants.smooth_proxy(NprSmoothProxyPolicy {
            levels: 0,
            weld_relative_tolerance: 0.0,
            ..NprSmoothProxyPolicy::default()
        }).unwrap().geometry().vertices.len();
        let tolerant = variants.smooth_proxy(NprSmoothProxyPolicy {
            levels: 0,
            weld_relative_tolerance: 1.0e-5,
            ..NprSmoothProxyPolicy::default()
        }).unwrap().geometry().vertices.len();
        assert_eq!(exact, 6);
        assert_eq!(tolerant, 4);
    }

    #[test]
    fn source_anchor_resolves_in_local_space_and_survives_object_motion() {
        let surface = NprPreparedSurface::new(NprGeometry::canonical_cube());
        let anchor = surface.anchor(0, [0.2, 0.3, 0.5]).unwrap();
        let sample = surface.sample(anchor).unwrap();
        let [a, b, c] = surface.geometry().triangles[0]
            .map(|index| surface.geometry().vertices[index as usize].position);
        assert_eq!(sample.position, a * 0.2 + b * 0.3 + c * 0.5);
        assert!(sample.normal.is_finite());

        let object_transform = Mat4::from_scale_rotation_translation(
            Vec3::splat(1.5),
            glam::Quat::from_rotation_y(0.7),
            Vec3::new(2.0, -1.0, 3.0),
        );
        assert_eq!(
            object_transform.transform_point3(sample.position),
            object_transform.transform_point3(a * 0.2 + b * 0.3 + c * 0.5)
        );
    }

    #[test]
    fn source_anchor_rejects_invalid_coordinates_and_other_revisions() {
        let cube = NprPreparedSurface::new(NprGeometry::canonical_cube());
        let wedge = NprPreparedSurface::new(NprGeometry::wedge());
        assert_eq!(
            cube.anchor(0, [0.5, 0.5, 0.5]),
            Err(NprSurfaceAnchorError::InvalidBarycentric)
        );
        assert_eq!(
            cube.anchor(99, [1.0, 0.0, 0.0]),
            Err(NprSurfaceAnchorError::TriangleOutOfRange)
        );
        assert_eq!(
            wedge.sample(cube.anchor(0, [1.0, 0.0, 0.0]).unwrap()),
            Err(NprSurfaceAnchorError::ContentMismatch)
        );
    }

    #[test]
    fn raycast_returns_the_nearest_source_anchor_in_local_space() {
        let surface = NprPreparedSurface::new(NprGeometry::canonical_cube());
        let hit = surface
            .raycast(Vec3::new(0.25, 0.2, 3.0), -Vec3::Z)
            .expect("ray should hit the front cube face before the back face");

        assert_eq!(hit.anchor.content_id, surface.content_id());
        assert_eq!(hit.anchor.triangle, 2);
        assert!((hit.distance - 2.0).abs() < 1e-6);
        assert!((hit.position - Vec3::new(0.25, 0.2, 1.0)).length() < 1e-6);
        assert!(hit.normal.dot(Vec3::Z) > 0.9999);
        assert!((hit.anchor.barycentric.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(hit.anchor.barycentric.iter().all(|weight| *weight >= 0.0));
    }

    #[test]
    fn raycast_rejects_rays_that_cannot_hit_a_surface() {
        let surface = NprPreparedSurface::new(NprGeometry::canonical_cube());
        assert!(surface.raycast(Vec3::new(0.0, 0.0, 3.0), Vec3::Z).is_none());
        assert!(surface.raycast(Vec3::ZERO, Vec3::ZERO).is_none());
        assert!(surface.raycast(Vec3::NAN, -Vec3::Z).is_none());
    }

    #[test]
    fn smooth_proxy_charts_resolve_marks_to_the_source_revision() {
        let mut variants = NprPreparedSurfaceVariants::new(NprGeometry::icosphere());
        let source = variants.source().clone();
        let proxy = variants
            .smooth_proxy(NprSmoothProxyPolicy {
                levels: 1,
                ..NprSmoothProxyPolicy::default()
            })
            .unwrap();
        assert_ne!(proxy.content_id(), source.content_id());
        assert_eq!(proxy.source_content_id(), source.content_id());

        let anchor = proxy.source_anchor(3, [1.0 / 3.0; 3]).unwrap();
        assert_eq!(anchor.content_id, source.content_id());
        assert_eq!(anchor.triangle, 0);
        for weight in anchor.barycentric {
            assert!((weight - 1.0 / 3.0).abs() < 1e-6);
        }
        assert!(source.sample(anchor).unwrap().position.is_finite());
    }
}
