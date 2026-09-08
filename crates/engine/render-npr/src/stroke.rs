//! Deterministic feature-chain assembly, independent of projection and styling.
use crate::{FeatureClass, FeatureSegment};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureStroke {
    pub id: u32,
    pub class: FeatureClass,
    pub vertices: Vec<u32>,
}
pub fn chain_features(features: &[FeatureSegment]) -> Vec<FeatureStroke> {
    let mut adjacency: BTreeMap<(FeatureClass, u32), Vec<usize>> = BTreeMap::new();
    for (i, f) in features.iter().enumerate() {
        for v in [f.edge.a, f.edge.b] {
            adjacency.entry((f.class, v)).or_default().push(i);
        }
    }
    let mut remaining = (0..features.len()).collect::<BTreeSet<_>>();
    let mut strokes = Vec::new();
    while !remaining.is_empty() {
        // Consume open chains from their endpoint before any cycles. Starting at
        // an arbitrary interior edge would split a chain in two.
        let first = *remaining
            .iter()
            .find(|&&i| {
                let f = features[i];
                [f.edge.a, f.edge.b]
                    .iter()
                    .any(|v| adjacency[&(f.class, *v)].len() != 2)
            })
            .unwrap_or_else(|| remaining.iter().next().unwrap());
        let f = features[first];
        let start = [f.edge.a, f.edge.b]
            .into_iter()
            .find(|v| adjacency[&(f.class, *v)].len() != 2)
            .unwrap_or(f.edge.a);
        let mut vertices = vec![start];
        let mut edge = first;
        let mut current = start;
        let mut id = u32::MAX;
        loop {
            if !remaining.remove(&edge) {
                break;
            }
            let f = features[edge];
            id = id.min(f.edge.a.wrapping_mul(73856093) ^ f.edge.b.wrapping_mul(19349663));
            current = if f.edge.a == current {
                f.edge.b
            } else {
                f.edge.a
            };
            vertices.push(current);
            let adjacent = &adjacency[&(f.class, current)];
            if current == start || adjacent.len() != 2 {
                break;
            }
            let Some(next) = adjacent.iter().find(|i| remaining.contains(i)) else {
                break;
            };
            edge = *next;
        }
        strokes.push(FeatureStroke {
            id,
            class: f.class,
            vertices,
        });
    }
    strokes
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::TopologyEdge;
    #[test]
    fn joins_cycle_and_preserves_class_boundaries() {
        let edges = [(0, 1), (1, 2), (2, 0)].map(|(a, b)| FeatureSegment {
            edge: TopologyEdge {
                a,
                b,
                faces: [0, u32::MAX],
            },
            class: FeatureClass::Boundary,
            midpoint: glam::Vec3::ZERO,
        });
        let strokes = chain_features(&edges);
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].vertices.first(), strokes[0].vertices.last());
    }
}
