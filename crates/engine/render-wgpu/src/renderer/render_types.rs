use amigo_math::{ColorRgba, Transform3, Vec2};
use amigo_render_api::{ParticleBlendMode2dPrimitive, ParticleLineAnchor2dPrimitive};
use std::collections::BTreeMap;

use crate::renderer::vertices::{ColorVertex, TextureVertex};

pub(crate) type ParticleBlendMode2d = ParticleBlendMode2dPrimitive;
pub(crate) type ParticleLineAnchor2d = ParticleLineAnchor2dPrimitive;

#[derive(Clone, Copy)]
pub(crate) struct ProjectedPoint {
    pub(crate) position: Vec2,
    pub(crate) depth: f32,
}

#[derive(Clone)]
pub(crate) struct ProjectedTriangle {
    pub(crate) depths: [f32; 3],
    pub(crate) points: [Vec2; 3],
    pub(crate) color: ColorRgba,
    pub(crate) depth: f32,
    pub(crate) render_order: i32,
    pub(crate) hatch: Option<ProjectedTriangleHatch>,
    pub(crate) ink: Option<ProjectedTriangleInk>,
}

#[derive(Clone, Copy)]
pub(crate) struct ProjectedTriangleHatch {
    pub(crate) surface_coordinates: [Vec2; 3],
    pub(crate) depths: [f32; 3],
    pub(crate) pencil_grain: f32,
    pub(crate) wobble_pixels: f32,
    pub(crate) stroke_segments: u8,
    pub(crate) pressure_variation: f32,
    pub(crate) hardness: f32,
    pub(crate) dryness: f32,
    pub(crate) color: ColorRgba,
    pub(crate) angle_degrees: f32,
    pub(crate) spacing_pixels: f32,
    pub(crate) cross: f32,
    pub(crate) width_pixels: f32,
    pub(crate) seed: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct ProjectedTriangleInk {
    pub(crate) edges: [Option<ProjectedInkEdge>; 3],
}

#[derive(Clone, Copy)]
pub(crate) struct ProjectedInkEdge {
    pub(crate) depths: [f32; 2],
    pub(crate) color: ColorRgba,
    pub(crate) width_pixels: f32,
    pub(crate) wobble_pixels: f32,
    pub(crate) stroke_segments: u8,
    pub(crate) pressure_variation: f32,
    /// Normalized position of this edge inside a joined gesture. Standalone
    /// edges use 0..1; chained contour segments share one continuous phase.
    pub(crate) gesture_t0: f32,
    pub(crate) gesture_t1: f32,
    pub(crate) taper: f32,
    pub(crate) overstroke: f32,
    pub(crate) hardness: f32,
    pub(crate) dryness: f32,
    pub(crate) pencil_grain: f32,
    pub(crate) persistent: bool,
    pub(crate) stable_seed: u64,
    pub(crate) seed: u64,
}

#[derive(Default)]
pub(crate) struct NprStrokeHistory {
    entries: BTreeMap<(u64, u32), NprStrokeHistoryEntry>,
    generation: u64,
    last_camera: Option<Transform3>,
}

#[derive(Clone, Copy)]
struct NprStrokeHistoryEntry {
    points: [Vec2; 2],
    style: ProjectedInkEdge,
    depth: f32,
    generation: u64,
    artistic_frame: u64,
}

impl NprStrokeHistory {
    /// Cached history is stored in screen space. Any camera movement invalidates
    /// it, preventing a previous contour from appearing at the old projection.
    pub(crate) fn observe_camera(&mut self, camera: Transform3) {
        let changed = self.last_camera.is_some_and(|previous| {
            let distance = |a: amigo_math::Vec3, b: amigo_math::Vec3| {
                (a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)
            };
            let delta = distance(previous.translation, camera.translation)
                + distance(previous.rotation_euler, camera.rotation_euler)
                + distance(previous.scale, camera.scale);
            delta > 1.0e-8
        });
        if changed {
            self.entries.clear();
        }
        self.last_camera = Some(camera);
    }

    pub(crate) fn begin_frame(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.entries
            .retain(|_, entry| self.generation.saturating_sub(entry.generation) < 240);
    }

    pub(crate) fn remember(
        &mut self,
        entity_id: u64,
        edge_index: u32,
        points: [Vec2; 2],
        style: ProjectedInkEdge,
        depth: f32,
        artistic_frame: u64,
    ) {
        self.entries.insert(
            (entity_id, edge_index),
            NprStrokeHistoryEntry {
                points,
                style,
                depth,
                generation: self.generation,
                artistic_frame,
            },
        );
    }

    pub(crate) fn persistent_for(
        &mut self,
        entity_id: u64,
        artistic_frame: u64,
    ) -> Vec<([Vec2; 2], ProjectedInkEdge, f32)> {
        let mut expired = Vec::new();
        let mut retained = Vec::new();
        for (&(cached_entity, edge_index), entry) in
            self.entries.range((entity_id, 0)..=(entity_id, u32::MAX))
        {
            if entry.generation == self.generation {
                continue;
            }
            let age = artistic_frame.saturating_sub(entry.artistic_frame);
            if age <= 2 {
                let mut style = entry.style;
                style.color.a *= 1.0 - age as f32 * 0.28;
                retained.push((entry.points, style, entry.depth));
            } else {
                expired.push((cached_entity, edge_index));
            }
        }
        for key in expired {
            self.entries.remove(&key);
        }
        retained
    }
}

#[cfg(test)]
mod stroke_history_tests {
    use super::*;

    fn edge() -> ProjectedInkEdge {
        ProjectedInkEdge {
            depths: [1.0; 2],
            color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            width_pixels: 1.0,
            wobble_pixels: 1.0,
            stroke_segments: 3,
            pressure_variation: 0.2,
            gesture_t0: 0.0,
            gesture_t1: 1.0,
            taper: 0.1,
            overstroke: 0.1,
            hardness: 0.5,
            pencil_grain: 0.4,
                dryness: 0.1,
                persistent: true,
                stable_seed: 7,
            seed: 9,
        }
    }

    #[test]
    fn stroke_history_holds_a_missing_edge_for_two_artistic_frames() {
        let mut history = NprStrokeHistory::default();
        history.begin_frame();
        history.remember(4, 2, [Vec2::ZERO, Vec2::new(1.0, 0.0)], edge(), 0.5, 10);
        history.begin_frame();
        assert_eq!(history.persistent_for(4, 11).len(), 1);
        history.begin_frame();
        assert_eq!(history.persistent_for(4, 12).len(), 1);
        history.begin_frame();
        assert!(history.persistent_for(4, 13).is_empty());
    }

    #[test]
    fn stroke_history_is_invalidated_when_camera_projection_moves() {
        let mut history = NprStrokeHistory::default();
        history.observe_camera(Transform3::default());
        history.begin_frame();
        history.remember(4, 2, [Vec2::ZERO, Vec2::new(1.0, 0.0)], edge(), 0.5, 10);
        history.observe_camera(Transform3 {
            translation: amigo_math::Vec3::new(0.01, 0.0, 0.0),
            ..Transform3::default()
        });
        assert!(history.persistent_for(4, 11).is_empty());
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TextureUvRect {
    pub(crate) u0: f32,
    pub(crate) v0: f32,
    pub(crate) u1: f32,
    pub(crate) v1: f32,
}

#[derive(Clone)]
pub(crate) struct TextureBatch {
    pub(crate) blend_mode: TextureBlendMode,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) _owned_sampler: Option<wgpu::Sampler>,
    pub(crate) vertices: Vec<TextureVertex>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextureBlendMode {
    Opaque,
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

#[derive(Clone)]
pub(crate) struct ColorBatch {
    pub(crate) world_depth: bool,
    pub(crate) depth_write: bool,
    pub(crate) blend_mode: ParticleBlendMode2d,
    pub(crate) pencil: bool,
    pub(crate) vertices: Vec<ColorVertex>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpriteSheet {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: Vec2,
    pub fps: f32,
    pub looping: bool,
}

impl SpriteSheet {
    pub(crate) fn visible_frame_count(&self) -> u32 {
        self.frame_count
            .max(1)
            .min(self.columns.max(1).saturating_mul(self.rows.max(1)))
    }
}
