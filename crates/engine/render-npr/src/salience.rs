//! Drawing-intent ranking for geometric feature candidates.

use crate::{FeatureClass, FeatureSegment, NprCamera, NprGeometry, face_normal};
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprSaliencePolicy {
    pub minimum_score: f32,
    pub crease_length_px: f32,
    pub crease_weight: f32,
}

impl Default for NprSaliencePolicy {
    fn default() -> Self {
        Self { minimum_score: 0.28, crease_length_px: 12.0, crease_weight: 0.65 }
    }
}

/// The result is separate from `FeatureSegment`: detection says what exists,
/// salience says what should be drawn in this image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprFeatureSalience {
    pub feature: FeatureSegment,
    pub score: f32,
}

pub fn evaluate_feature_salience(
    geometry: &NprGeometry,
    camera: NprCamera,
    viewport: Vec2,
    feature: FeatureSegment,
    policy: NprSaliencePolicy,
) -> Option<NprFeatureSalience> {
    let (a, b) = camera.project_segment(
        geometry.vertices[feature.edge.a as usize].position,
        geometry.vertices[feature.edge.b as usize].position,
        viewport,
    )?;
    let length_score = (a.screen.distance(b.screen) / policy.crease_length_px).min(1.0);
    let score = match feature.class {
        FeatureClass::Silhouette | FeatureClass::Boundary => 1.0,
        FeatureClass::Crease => {
            let first = face_normal(geometry, feature.edge.faces[0]);
            let second = face_normal(geometry, feature.edge.faces[1]);
            let crease_strength = (1.0 - first.dot(second)).clamp(0.0, 1.0);
            length_score * (0.35 + policy.crease_weight * crease_strength)
        }
        // Hatching is authored by the mark planner, never supplied as an edge
        // detector candidate.
        FeatureClass::Hatching => 0.0,
    };
    Some(NprFeatureSalience { feature, score })
}

pub fn select_salient_features(
    geometry: &NprGeometry,
    camera: NprCamera,
    viewport: Vec2,
    features: &[FeatureSegment],
    policy: NprSaliencePolicy,
) -> Vec<FeatureSegment> {
    features
        .iter()
        .copied()
        .filter_map(|feature| evaluate_feature_salience(geometry, camera, viewport, feature, policy))
        .filter(|rank| rank.score >= policy.minimum_score)
        .map(|rank| rank.feature)
        .collect()
}
