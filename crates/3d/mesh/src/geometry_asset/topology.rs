use super::pose::Primitive;
use glam::Vec3;
use std::collections::BTreeMap;

/// Weld only equivalent deformation signatures, once at import. Rewelding each frame
/// would change triangle identities and invalidate surface-attached authoring marks.
pub(super) fn weld(primitive: &mut Primitive) -> Result<usize, String> {
    let min = primitive
        .positions
        .iter()
        .copied()
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = primitive
        .positions
        .iter()
        .copied()
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    let epsilon = (max - min).length().max(1e-6) * 1e-6;
    let deformable = primitive.skin.is_some() || !primitive.morph_targets.is_empty();
    let mut welded = BTreeMap::new();
    let mut remap = Vec::new();
    let mut representatives = Vec::new();
    for (index, point) in primitive.positions.iter().enumerate() {
        let mut key = point
            .to_array()
            .map(|v| {
                if deformable {
                    v.to_bits() as i64
                } else {
                    (v / epsilon).round() as i64
                }
            })
            .to_vec();
        for target in &primitive.morph_targets {
            key.extend(target[index].to_array().map(|v| v.to_bits() as i64));
        }
        for &(joint, weight) in &primitive.influences[index] {
            key.extend([joint as i64, weight.to_bits() as i64]);
        }
        let next = representatives.len() as u32;
        let mapped = *welded.entry(key).or_insert_with(|| {
            representatives.push(index);
            next
        });
        remap.push(mapped);
    }
    let positions = representatives
        .iter()
        .map(|&i| primitive.positions[i])
        .collect::<Vec<_>>();
    let mut indices = Vec::new();
    let mut dropped = 0;
    for tri in primitive.indices.chunks_exact(3) {
        let t = [
            remap[tri[0] as usize],
            remap[tri[1] as usize],
            remap[tri[2] as usize],
        ];
        let [a, b, c] = t.map(|i| positions[i as usize]);
        if t[0] == t[1]
            || t[1] == t[2]
            || t[2] == t[0]
            || (!deformable && (b - a).cross(c - a).length_squared() <= epsilon.powi(4))
        {
            dropped += 1;
            continue;
        }
        // A glTF mesh remains valid render input when more than two faces
        // share an edge. Rejecting it here made ordinary third-party city
        // assets disappear before either Mesh3D or NPR could prepare them.
        // Consumers that need a closed manifold (for example a topology tool)
        // must validate that requirement explicitly at their own boundary.
        indices.extend(t);
    }
    primitive.positions = positions;
    primitive.indices = indices;
    primitive.morph_targets = primitive
        .morph_targets
        .iter()
        .map(|target| representatives.iter().map(|&i| target[i]).collect())
        .collect();
    primitive.influences = representatives
        .iter()
        .map(|&i| primitive.influences[i].clone())
        .collect();
    Ok(dropped)
}
