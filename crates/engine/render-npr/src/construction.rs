//! Authored construction marks anchored to an immutable drawing surface.
//!
//! A construction mark deliberately stores no world-space points.  Scene or
//! editor code can keep these values across object transforms and a prepared
//! smooth proxy, while the renderer validates the source revision before it
//! resolves and draws the mark.

use crate::NprSurfaceAnchor;

#[derive(Debug, Clone, PartialEq)]
pub struct NprConstructionMark {
    /// Domain-owned stable identity. Reserve a domain-specific range when a
    /// mark is generated rather than manually authored.
    pub id: u32,
    pub anchors: Vec<NprSurfaceAnchor>,
    pub closed: bool,
    /// Relative to authored crease ink, in pixel space.
    pub width_scale: f32,
    /// Multiplies the resolved stroke coverage without changing geometry.
    pub opacity: f32,
}

impl NprConstructionMark {
    pub fn new(id: u32, anchors: Vec<NprSurfaceAnchor>) -> Self {
        Self {
            id,
            anchors,
            closed: false,
            width_scale: 0.5,
            opacity: 0.35,
        }
    }
}
