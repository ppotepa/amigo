use amigo_math::Vec3;

pub(crate) fn gpu_candidate_strategy(value: amigo_render_api::NprCandidateStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprCandidateStrategy3d::GeometryEdges => 0,
        amigo_render_api::NprCandidateStrategy3d::CharacterSemantic => 1,
    }
}

pub(crate) fn gpu_path_strategy(value: amigo_render_api::NprPathStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprPathStrategy3d::StableStrokedPaths => 0,
        amigo_render_api::NprPathStrategy3d::DirectVisibleSegments => 1,
    }
}

pub(crate) fn gpu_stroke_strategy(value: amigo_render_api::NprStrokeStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprStrokeStrategy3d::ComicInk => 0,
        amigo_render_api::NprStrokeStrategy3d::AkiraInk => 1,
        amigo_render_api::NprStrokeStrategy3d::TechnicalInk => 2,
        amigo_render_api::NprStrokeStrategy3d::RoughPencil => 3,
    }
}

pub(crate) fn gpu_fill_strategy(value: amigo_render_api::NprInkFillStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprInkFillStrategy3d::None => 0,
        amigo_render_api::NprInkFillStrategy3d::MaterialBlackMass => 1,
        amigo_render_api::NprInkFillStrategy3d::BinaryMangaShadow => 2,
    }
}

pub(crate) fn gpu_hatching_strategy(value: amigo_render_api::NprHatchingStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprHatchingStrategy3d::None => 0,
        amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching => 1,
    }
}

pub(crate) fn gpu_budget_strategy(value: amigo_render_api::NprBudgetStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprBudgetStrategy3d::EdgeVisibility => 0,
        amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority => 1,
        amigo_render_api::NprBudgetStrategy3d::CharacterReadability => 2,
    }
}

pub(crate) fn gpu_temporal_strategy(value: amigo_render_api::NprTemporalStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprTemporalStrategy3d::PathHistory => 0,
        amigo_render_api::NprTemporalStrategy3d::StableArcLength => 1,
    }
}

pub(crate) fn gpu_material_id_mask(material_ids: &[u32]) -> u32 {
    material_ids
        .iter()
        .filter(|id| **id < 32)
        .fold(0u32, |mask, id| mask | (1u32 << *id))
}

pub(crate) fn vec3_to_gpu4(value: Vec3, w: f32) -> [f32; 4] {
    [value.x, value.y, value.z, w]
}
