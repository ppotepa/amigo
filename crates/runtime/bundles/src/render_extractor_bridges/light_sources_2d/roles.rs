use super::*;

pub(super) fn light_source_roles(source: &LightSource2dCommon) -> RenderContributionSet {
    RenderContributionSet::from_pairs(source.contributions.iter().map(|kind| {
        let role = match kind {
            LightContributionKind2d::LightingEmit => {
                amigo_render_api::render_contribution_roles::LIGHTING_EMIT
            }
            LightContributionKind2d::RelightPlate => {
                amigo_render_api::render_contribution_roles::RELIGHT_PLATE
            }
            LightContributionKind2d::BloomSource => {
                amigo_render_api::render_contribution_roles::BLOOM_SOURCE
            }
            LightContributionKind2d::CameraFxSource => {
                amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE
            }
            LightContributionKind2d::EmissiveBuffer => "emissive_buffer",
        };
        (role, true)
    }))
}

pub(super) fn visual_source_availability_label(
    availability: VisualSourceAvailability2d,
) -> &'static str {
    match availability {
        VisualSourceAvailability2d::Produced => "produced",
        VisualSourceAvailability2d::Derived => "derived",
        VisualSourceAvailability2d::Asset => "asset",
        VisualSourceAvailability2d::Missing => "missing",
    }
}
