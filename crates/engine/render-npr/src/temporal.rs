//! Explicit, backend-independent temporal state for NPR strokes.
//!
//! A render packet remains a pure reference-frame result. `DrawingHistory` is
//! owned by a scene/session and only modulates the visibility of stable stroke
//! identities. This keeps frame-rate-dependent state out of tessellation and
//! out of the renderer.

use crate::{FeatureClass, NprRenderPacket};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit policy for whether a gesture is allowed to acquire a new seeded
/// variant while its projected surface moves. It does not affect projection,
/// visibility or surface anchors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrokeMotionMode {
    #[default]
    Stable,
    RedrawOnMotion,
}

/// Domain-owned motion policy. The host supplies projected, stable surface
/// anchors; this type deliberately has no window, renderer or frame-rate API.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprMotionPolicy {
    pub mode: StrokeMotionMode,
    /// Upper bound on gesture-variant changes while a motion gate is active.
    pub redraw_hz: f32,
    /// Fraction of a redraw variant visible in the gesture seed. Zero leaves
    /// the stable seed intact; one selects the full new variant.
    pub redraw_strength: f32,
    /// Time constant for a genuinely new visible stroke. Zero is immediate.
    pub appearance_fade_seconds: f32,
}

impl Default for NprMotionPolicy {
    fn default() -> Self {
        Self {
            mode: StrokeMotionMode::Stable,
            redraw_hz: 3.0,
            redraw_strength: 1.0,
            appearance_fade_seconds: 0.12,
        }
    }
}

#[derive(Debug, Clone)]
struct VariantState {
    anchors: Vec<Vec2>,
    epoch: u32,
    active: bool,
    seconds_until_next: f32,
}

/// Tracks a bounded redraw clock for stable drawing scopes. Motion is measured
/// from projected surface anchors, so rotating a symmetric bounding box cannot
/// accidentally look stationary.
#[derive(Debug, Default)]
pub struct StrokeVariantClock {
    scopes: BTreeMap<u64, VariantState>,
}

impl StrokeVariantClock {
    pub fn clear(&mut self) {
        self.scopes.clear();
    }

    pub fn reset_scope(&mut self, scope: u64) {
        self.scopes.remove(&scope);
    }

    /// Advances a scope exactly once for an extracted view frame. The returned
    /// epoch is zero in stable mode. Crossing the motion threshold redraws
    /// once immediately; continuous motion is then rate-limited by `redraw_hz`.
    pub fn advance(
        &mut self,
        scope: u64,
        anchors: &[Vec2],
        delta_seconds: f32,
        policy: NprMotionPolicy,
    ) -> u32 {
        let anchors = anchors.to_vec();
        let Some(previous) = self.scopes.get_mut(&scope) else {
            self.scopes.insert(
                scope,
                VariantState {
                    anchors,
                    epoch: 0,
                    active: false,
                    seconds_until_next: 0.0,
                },
            );
            return 0;
        };

        let motion_pixels = anchor_motion_pixels(&previous.anchors, &anchors);
        previous.anchors = anchors;
        if policy.mode == StrokeMotionMode::Stable || policy.redraw_strength <= 0.0 {
            previous.epoch = 0;
            previous.active = false;
            previous.seconds_until_next = 0.0;
            return previous.epoch;
        }

        // Hysteresis stops a slow orbit from toggling redraw on neighbouring
        // pixels. A path must move 0.75 px to enter and may settle below 0.35.
        let entering = !previous.active && motion_pixels >= 0.75;
        previous.active = if previous.active {
            motion_pixels >= 0.35
        } else {
            entering
        };
        if entering {
            previous.epoch = previous.epoch.wrapping_add(1);
            previous.seconds_until_next = redraw_period(policy.redraw_hz);
        } else if previous.active {
            previous.seconds_until_next -= delta_seconds.clamp(0.0, 0.25);
            let period = redraw_period(policy.redraw_hz);
            while previous.seconds_until_next <= 0.0 {
                previous.epoch = previous.epoch.wrapping_add(1);
                previous.seconds_until_next += period;
            }
        } else {
            previous.seconds_until_next = 0.0;
        }
        previous.epoch
    }
}

fn redraw_period(redraw_hz: f32) -> f32 {
    1.0 / redraw_hz.clamp(0.25, 20.0)
}

fn anchor_motion_pixels(previous: &[Vec2], current: &[Vec2]) -> f32 {
    let count = previous.len().min(current.len());
    if count == 0 {
        return 0.0;
    }
    (previous
        .iter()
        .zip(current)
        .take(count)
        .map(|(a, b)| a.distance_squared(*b))
        .sum::<f32>()
        / count as f32)
        .sqrt()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TemporalPolicy {
    /// Time constant for a stroke which has just entered an already-established
    /// drawing scope. A zero value deliberately means immediate visibility.
    pub appear_seconds: f32,
    /// How long an absent identity is retained for a possible reappearance.
    /// Absent geometry is never submitted as a stale screen-space ghost.
    pub retention_seconds: f32,
}

impl Default for TemporalPolicy {
    fn default() -> Self {
        Self {
            appear_seconds: 0.12,
            retention_seconds: 0.35,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TemporalStrokeKey {
    scope: u64,
    id: u32,
    class: FeatureClass,
    role: crate::StrokeRole,
    correction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TemporalStrokeState {
    visibility: f32,
    last_seen: u64,
    absent_seconds: f32,
}

/// Per-call information useful to diagnostics without exposing internal maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TemporalAdvanceStats {
    pub retained_strokes: usize,
    pub entering_strokes: usize,
}

/// Stateful continuity controller. Scopes are supplied by the domain owner;
/// changing a profile, seed or model should select a new scope or reset one.
#[derive(Debug, Default)]
pub struct DrawingHistory {
    scopes: BTreeSet<u64>,
    strokes: BTreeMap<TemporalStrokeKey, TemporalStrokeState>,
    frame: u64,
}

impl DrawingHistory {
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.strokes.clear();
        self.frame = 0;
    }

    pub fn reset_scope(&mut self, scope: u64) {
        self.scopes.remove(&scope);
        self.strokes.retain(|key, _| key.scope != scope);
    }

    /// Starts one logical view frame. A gallery may contribute several packets
    /// to that frame; it must not age its history once per object.
    pub fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    /// Applies continuity to one packet belonging to the current logical view
    /// frame. Call [`Self::begin_frame`] once before all packets and
    /// [`Self::finish_frame`] once afterwards.
    pub fn advance_packet_in_frame(
        &mut self,
        scope: u64,
        packet: &mut NprRenderPacket,
        delta_seconds: f32,
        policy: TemporalPolicy,
    ) -> TemporalAdvanceStats {
        let frame = self.frame;
        let first_scope_packet = self.scopes.insert(scope);
        let delta_seconds = delta_seconds.clamp(0.0, 0.25);
        let appear_seconds = policy.appear_seconds.max(0.0);
        let mut stats = TemporalAdvanceStats::default();

        for stroke in &mut packet.strokes {
            let key = TemporalStrokeKey {
                scope,
                id: stroke.id,
                class: stroke.class,
                role: stroke.role,
                correction: stroke.correction,
            };
            let visibility = match self.strokes.get_mut(&key) {
                Some(state) => {
                    stats.retained_strokes += 1;
                    state.visibility =
                        approach_one(state.visibility, delta_seconds, appear_seconds);
                    state.last_seen = frame;
                    state.absent_seconds = 0.0;
                    state.visibility
                }
                None => {
                    stats.entering_strokes += 1;
                    let visibility = if first_scope_packet {
                        1.0
                    } else {
                        approach_one(0.0, delta_seconds, appear_seconds)
                    };
                    self.strokes.insert(
                        key,
                        TemporalStrokeState {
                            visibility,
                            last_seen: frame,
                            absent_seconds: 0.0,
                        },
                    );
                    visibility
                }
            };
            for vertex in &mut stroke.vertices {
                vertex.coverage *= visibility;
            }
        }

        packet.stats.temporal_retained_strokes = stats.retained_strokes;
        packet.stats.temporal_entering_strokes = stats.entering_strokes;
        stats
    }

    /// Ages identities absent from the complete logical view frame. This keeps
    /// retention independent of the number and ordering of rendered objects.
    pub fn finish_frame(&mut self, delta_seconds: f32, policy: TemporalPolicy) {
        let frame = self.frame;
        let delta_seconds = delta_seconds.clamp(0.0, 0.25);
        let retention_seconds = policy.retention_seconds.max(0.0);
        self.strokes.retain(|_, state| {
            if state.last_seen != frame {
                state.absent_seconds += delta_seconds;
            }
            state.absent_seconds <= retention_seconds
        });
        // A profile may produce a new scope repeatedly during a long workshop
        // session. Do not retain empty scope markers forever.
        self.scopes
            .retain(|scope| self.strokes.keys().any(|key| key.scope == *scope));
    }

    /// Convenience entry point for a single-packet view frame. Multi-packet
    /// callers should use the explicit begin/advance/finish lifecycle above.
    pub fn advance_packet(
        &mut self,
        scope: u64,
        packet: &mut NprRenderPacket,
        delta_seconds: f32,
        policy: TemporalPolicy,
    ) -> TemporalAdvanceStats {
        self.begin_frame();
        let stats = self.advance_packet_in_frame(scope, packet, delta_seconds, policy);
        self.finish_frame(delta_seconds, policy);
        stats
    }
}

fn approach_one(value: f32, delta_seconds: f32, time_constant: f32) -> f32 {
    if time_constant <= f32::EPSILON {
        1.0
    } else {
        value + (1.0 - value) * (1.0 - (-delta_seconds / time_constant).exp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NprDebugView, NprRenderStats, StrokeVertex, TessellatedStroke};
    use glam::{Vec2, Vec4};

    fn packet(ids: &[u32]) -> NprRenderPacket {
        NprRenderPacket {
            occluders: vec![],
            fills: vec![],
            strokes: ids
                .iter()
                .map(|&id| TessellatedStroke {
                    id,
                    class: FeatureClass::Crease,
                    vertices: vec![StrokeVertex {
                        position: Vec2::ZERO,
                        width: 1.0,
                        id,
                        depth: 0.5,
                        pressure: 1.0,
                        coverage: 1.0,
                        grain: 0.0,
                        edge: 0.0,
                        edge_softness: 0.0,
                        paper_tooth: 0.0,
                        dryness: 0.0,
                    }],
                    ..Default::default()
                })
                .collect(),
            background: Vec4::ONE,
            debug_view: NprDebugView::Final,
            ink: Vec4::ONE,
            stats: NprRenderStats::default(),
        }
    }

    #[test]
    fn first_packet_is_visible_and_later_identity_enters_smoothly() {
        let mut history = DrawingHistory::default();
        let mut first = packet(&[1]);
        history.advance_packet(7, &mut first, 1.0 / 60.0, TemporalPolicy::default());
        assert_eq!(first.strokes[0].vertices[0].coverage, 1.0);

        let mut second = packet(&[1, 2]);
        let stats = history.advance_packet(7, &mut second, 0.06, TemporalPolicy::default());
        assert_eq!(stats.retained_strokes, 1);
        assert_eq!(stats.entering_strokes, 1);
        assert_eq!(second.strokes[0].vertices[0].coverage, 1.0);
        assert!(second.strokes[1].vertices[0].coverage > 0.0);
        assert!(second.strokes[1].vertices[0].coverage < 1.0);
    }

    #[test]
    fn reset_scope_has_no_blank_frame() {
        let mut history = DrawingHistory::default();
        let mut first = packet(&[1]);
        history.advance_packet(9, &mut first, 0.01, TemporalPolicy::default());
        history.reset_scope(9);
        let mut reset = packet(&[2]);
        history.advance_packet(9, &mut reset, 0.01, TemporalPolicy::default());
        assert_eq!(reset.strokes[0].vertices[0].coverage, 1.0);
    }

    #[test]
    fn fade_is_based_on_elapsed_time_not_frame_count() {
        let policy = TemporalPolicy::default();
        let mut sixty = DrawingHistory::default();
        let mut initial = packet(&[1]);
        sixty.advance_packet(1, &mut initial, 0.0, policy);
        let mut at_sixty = packet(&[1, 2]);
        sixty.advance_packet(1, &mut at_sixty, 0.1, policy);

        let mut thirty = DrawingHistory::default();
        let mut initial = packet(&[1]);
        thirty.advance_packet(1, &mut initial, 0.0, policy);
        let mut at_thirty = packet(&[1, 2]);
        thirty.advance_packet(1, &mut at_thirty, 0.1, policy);
        assert_eq!(
            at_sixty.strokes[1].vertices[0].coverage,
            at_thirty.strokes[1].vertices[0].coverage
        );
    }

    #[test]
    fn packets_in_one_view_frame_do_not_age_each_other() {
        let policy = TemporalPolicy {
            appear_seconds: 0.0,
            retention_seconds: 0.35,
        };
        let mut history = DrawingHistory::default();
        history.begin_frame();
        for scope in [1, 2, 3] {
            let mut frame = packet(&[scope]);
            history.advance_packet_in_frame(u64::from(scope), &mut frame, 0.2, policy);
        }
        history.finish_frame(0.2, policy);

        history.begin_frame();
        let mut only_first = packet(&[1]);
        history.advance_packet_in_frame(1, &mut only_first, 0.2, policy);
        history.finish_frame(0.2, policy);

        history.begin_frame();
        let mut returned = packet(&[2]);
        let stats = history.advance_packet_in_frame(2, &mut returned, 0.2, policy);
        history.finish_frame(0.2, policy);
        assert_eq!(stats.retained_strokes, 1);
        assert_eq!(stats.entering_strokes, 0);
    }

    #[test]
    fn stable_motion_mode_never_changes_the_gesture_epoch() {
        let mut clock = StrokeVariantClock::default();
        let policy = NprMotionPolicy::default();
        assert_eq!(clock.advance(3, &[Vec2::ZERO], 0.0, policy), 0);
        assert_eq!(clock.advance(3, &[Vec2::new(8.0, 0.0)], 1.0, policy), 0);
    }

    #[test]
    fn redraw_motion_mode_is_anchor_driven_and_rate_limited() {
        let mut clock = StrokeVariantClock::default();
        let policy = NprMotionPolicy {
            mode: StrokeMotionMode::RedrawOnMotion,
            redraw_hz: 4.0,
            ..Default::default()
        };
        assert_eq!(clock.advance(3, &[Vec2::ZERO], 0.0, policy), 0);
        // Crossing the gate creates one immediate redraw.
        assert_eq!(clock.advance(3, &[Vec2::new(1.0, 0.0)], 0.0, policy), 1);
        // Continued motion below one 250 ms period does not depend on FPS.
        assert_eq!(clock.advance(3, &[Vec2::new(2.0, 0.0)], 0.10, policy), 1);
        assert_eq!(clock.advance(3, &[Vec2::new(3.0, 0.0)], 0.15, policy), 2);
    }

    #[test]
    fn zero_redraw_strength_restores_the_stable_variant() {
        let mut clock = StrokeVariantClock::default();
        let policy = NprMotionPolicy {
            mode: StrokeMotionMode::RedrawOnMotion,
            redraw_strength: 0.0,
            ..Default::default()
        };
        assert_eq!(clock.advance(3, &[Vec2::ZERO], 0.0, policy), 0);
        assert_eq!(clock.advance(3, &[Vec2::new(8.0, 0.0)], 1.0, policy), 0);
    }
}
