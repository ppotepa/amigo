use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VisualSourceBufferResolutionPolicy {
    Skip,
    Half,
    Full,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct VisualSourceBufferPolicySet {
    pub layer_mask: VisualSourceBufferResolutionPolicy,
    pub layer_roles: VisualSourceBufferResolutionPolicy,
    pub scene_normal: VisualSourceBufferResolutionPolicy,
    pub scene_wetness: VisualSourceBufferResolutionPolicy,
    pub scene_highlight: VisualSourceBufferResolutionPolicy,
    pub scene_emissive: VisualSourceBufferResolutionPolicy,
    pub scene_motion: VisualSourceBufferResolutionPolicy,
}

impl VisualSourceBufferPolicySet {
    pub fn from_request(request: &WgpuFrameRenderRequest<'_>) -> Self {
        Self {
            layer_mask: layer_mask_policy(request),
            layer_roles: layer_roles_policy(request),
            scene_normal: source_policy(request, amigo_render_api::VisualSourceKind2d::SceneNormal),
            scene_wetness: source_policy(
                request,
                amigo_render_api::VisualSourceKind2d::SceneWetness,
            ),
            scene_highlight: source_policy(
                request,
                amigo_render_api::VisualSourceKind2d::SceneHighlight,
            ),
            scene_emissive: source_policy(
                request,
                amigo_render_api::VisualSourceKind2d::SceneEmissive,
            ),
            scene_motion: source_policy(request, amigo_render_api::VisualSourceKind2d::SceneMotion),
        }
    }
}

impl VisualSourceBufferResolutionPolicy {
    pub fn should_produce(self) -> bool {
        !matches!(self, Self::Skip)
    }
}

fn layer_mask_policy(request: &WgpuFrameRenderRequest<'_>) -> VisualSourceBufferResolutionPolicy {
    if matches!(
        request.camera_debug_view,
        amigo_render_api::CameraDebugView2d::LayerMask
    ) {
        return VisualSourceBufferResolutionPolicy::Full;
    }
    if request
        .visual_source_flags_2d
        .is_some_and(|flags| flags.layer_mask_generated)
    {
        VisualSourceBufferResolutionPolicy::Half
    } else {
        VisualSourceBufferResolutionPolicy::Skip
    }
}

fn layer_roles_policy(request: &WgpuFrameRenderRequest<'_>) -> VisualSourceBufferResolutionPolicy {
    if matches!(
        request.camera_debug_view,
        amigo_render_api::CameraDebugView2d::LayerOpticalRoles
    ) {
        return VisualSourceBufferResolutionPolicy::Full;
    }
    if request
        .visual_source_flags_2d
        .is_some_and(|flags| flags.layer_roles_generated)
    {
        VisualSourceBufferResolutionPolicy::Full
    } else {
        VisualSourceBufferResolutionPolicy::Skip
    }
}

fn source_policy(
    request: &WgpuFrameRenderRequest<'_>,
    kind: amigo_render_api::VisualSourceKind2d,
) -> VisualSourceBufferResolutionPolicy {
    let debug_wants = debug_view_wants_source(request.camera_debug_view, kind);
    let produced = request
        .camera_capture_input_2d
        .and_then(|input| input.source(kind))
        .is_some_and(|source| {
            source.availability == amigo_render_api::VisualSourceAvailability2d::Produced
        });

    if debug_wants {
        return VisualSourceBufferResolutionPolicy::Full;
    }

    if !produced {
        return VisualSourceBufferResolutionPolicy::Skip;
    }

    // V1: request currently receives resolved flags, so flags are the cost policy
    // until CameraQualitySettings2d is carried directly into the render request.
    let generated = request
        .visual_source_flags_2d
        .is_some_and(|flags| match kind {
            amigo_render_api::VisualSourceKind2d::SceneNormal => flags.scene_normal_generated,
            amigo_render_api::VisualSourceKind2d::SceneWetness => flags.scene_wetness_generated,
            amigo_render_api::VisualSourceKind2d::SceneHighlight => flags.scene_highlight_generated,
            amigo_render_api::VisualSourceKind2d::SceneEmissive => flags.scene_emissive_generated,
            amigo_render_api::VisualSourceKind2d::SceneMotion => flags.scene_motion_generated,
            amigo_render_api::VisualSourceKind2d::LayerMask => flags.layer_mask_generated,
            amigo_render_api::VisualSourceKind2d::SceneColor
            | amigo_render_api::VisualSourceKind2d::SceneDepth
            | amigo_render_api::VisualSourceKind2d::Debug => false,
        });

    if generated {
        VisualSourceBufferResolutionPolicy::Full
    } else {
        VisualSourceBufferResolutionPolicy::Skip
    }
}

fn debug_view_wants_source(
    debug_view: amigo_render_api::CameraDebugView2d,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    matches!(
        (debug_view, kind),
        (
            amigo_render_api::CameraDebugView2d::SceneNormals,
            amigo_render_api::VisualSourceKind2d::SceneNormal
        ) | (
            amigo_render_api::CameraDebugView2d::SceneWetness,
            amigo_render_api::VisualSourceKind2d::SceneWetness
        ) | (
            amigo_render_api::CameraDebugView2d::SceneHighlights,
            amigo_render_api::VisualSourceKind2d::SceneHighlight
        ) | (
            amigo_render_api::CameraDebugView2d::SceneEmissive,
            amigo_render_api::VisualSourceKind2d::SceneEmissive
        ) | (
            amigo_render_api::CameraDebugView2d::SceneMotion,
            amigo_render_api::VisualSourceKind2d::SceneMotion
        )
    )
}
