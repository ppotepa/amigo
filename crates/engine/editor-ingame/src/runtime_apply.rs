use amigo_2d_composition::RenderDepthMode2d;
use amigo_2d_post_fx::{PostFx2d, PostFx2dService, RainGlass2d, RainGlassDebugView};
use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_authoring::AuthoringRuntimeBinding;
use amigo_runtime::Runtime;

use crate::state::{EditorPropertyValue, IngameEditorState};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ApplyResult {
    Applied,
    MockApplied,
    Readonly,
    Unsupported,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ApplyRequest<'a> {
    pub property_id: &'a str,
    pub target: Option<&'a AuthoringRuntimeBinding>,
    pub previous: Option<EditorPropertyValue>,
    pub next: EditorPropertyValue,
}

pub fn apply_property_value(
    runtime: &Runtime,
    state: &IngameEditorState,
    property_id: &str,
    target: Option<&AuthoringRuntimeBinding>,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    apply_property_request(
        runtime,
        state,
        ApplyRequest {
            property_id,
            target,
            previous: None,
            next: value,
        },
    )
}

pub fn apply_property_request(
    runtime: &Runtime,
    state: &IngameEditorState,
    request: ApplyRequest<'_>,
) -> AmigoResult<ApplyResult> {
    let Some(target) = request.target else {
        state.set_status(format!("{}: unsupported", request.property_id));
        return Ok(ApplyResult::Unsupported);
    };

    let previous_label = request
        .previous
        .as_ref()
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "<unknown>".to_owned());
    let next_label = format!("{:?}", request.next);
    let result = match target {
        AuthoringRuntimeBinding::RenderLayerOpacity { layer_id }
        | AuthoringRuntimeBinding::RenderLayerVisible { layer_id }
        | AuthoringRuntimeBinding::RenderLayerOrder { layer_id }
        | AuthoringRuntimeBinding::RenderLayerDepthMode { layer_id }
        | AuthoringRuntimeBinding::RenderLayerZDepth { layer_id }
        | AuthoringRuntimeBinding::RenderLayerDistanceM { layer_id }
        | AuthoringRuntimeBinding::RenderLayerDepthBlurScale { layer_id } => {
            apply_render_layer_binding(runtime, target, layer_id, request.next)
        }
        AuthoringRuntimeBinding::LayeredImageBaseOpacity { .. }
        | AuthoringRuntimeBinding::LayeredImageLayerOpacity { .. }
        | AuthoringRuntimeBinding::LayeredImageLayerEnabled { .. } => {
            apply_layered_image_binding(runtime, target, request.next)
        }
        AuthoringRuntimeBinding::ParticleEmitterProperty { entity_name, field } => {
            apply_particle_property(runtime, entity_name, field, request.next)
        }
        AuthoringRuntimeBinding::PostFxFrameEnabled { index } => {
            let Some(value) = as_bool_value(&request.next) else {
                return Ok(ApplyResult::Unsupported);
            };
            let service = runtime.required::<PostFx2dService>()?;
            if service.set_frame_effect_enabled(*index, value) {
                Ok(ApplyResult::Applied)
            } else {
                Ok(ApplyResult::Failed(format!(
                    "postfx frame effect {index} not found"
                )))
            }
        }
        AuthoringRuntimeBinding::PostFxFrameField { index, field } => {
            apply_postfx_frame_field(runtime, *index, field, &request.next)
        }
        AuthoringRuntimeBinding::PostFxMock { .. } | AuthoringRuntimeBinding::Mock { .. } => {
            apply_mock_binding(state, request.property_id, request.next)
        }
    };

    match &result {
        Ok(ApplyResult::Applied) => state.set_status(format!(
            "{}: {previous_label} -> {next_label} [Applied Live]",
            request.property_id
        )),
        Ok(ApplyResult::Unsupported) => {
            state.set_status(format!("{}: unsupported", request.property_id))
        }
        Ok(ApplyResult::Readonly) => state.set_status(format!("{}: readonly", request.property_id)),
        Ok(ApplyResult::MockApplied) | Err(_) | Ok(ApplyResult::Failed(_)) => {}
    }
    result
}

fn apply_postfx_frame_field(
    runtime: &Runtime,
    index: usize,
    field: &str,
    value: &EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let service = runtime.required::<PostFx2dService>()?;
    let applied = service.update_frame_effect(index, |effect| match effect {
        PostFx2d::RainGlass(mut rain) => {
            if !apply_rain_glass_field(&mut rain, field, value) {
                return None;
            }
            Some(PostFx2d::RainGlass(rain.normalized()))
        }
        other => Some(other),
    });
    if applied {
        Ok(ApplyResult::Applied)
    } else {
        Ok(ApplyResult::Failed(format!(
            "unsupported postfx field `{field}`"
        )))
    }
}

fn apply_rain_glass_field(
    rain: &mut RainGlass2d,
    field: &str,
    value: &EditorPropertyValue,
) -> bool {
    match field {
        "opacity" => set_f32(value, &mut rain.opacity),
        "refract_scale" => set_f32(value, &mut rain.refract_scale),
        "background_blur_px" => set_f32(value, &mut rain.background_blur_px),
        "distortion_px" => set_f32(value, &mut rain.distortion_px),
        "normal_strength" => set_f32(value, &mut rain.normal_strength),
        "focus_blur_strength" => set_f32(value, &mut rain.focus_blur_strength),
        "body_opacity" => set_f32(value, &mut rain.body_opacity),
        "scene_blend" => set_f32(value, &mut rain.scene_blend),
        "mist_opacity" => set_f32(value, &mut rain.mist_opacity),
        "trails_enabled" => set_bool(value, &mut rain.trails_enabled),
        "mist_enabled" => set_bool(value, &mut rain.mist_enabled),
        "debug_view" => set_rain_glass_debug_view(value, rain),
        _ => false,
    }
}

fn as_bool_value(value: &EditorPropertyValue) -> Option<bool> {
    match value {
        EditorPropertyValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn set_f32(value: &EditorPropertyValue, out: &mut f32) -> bool {
    match value {
        EditorPropertyValue::Number(value) => {
            *out = *value;
            true
        }
        _ => false,
    }
}

fn set_bool(value: &EditorPropertyValue, out: &mut bool) -> bool {
    match value {
        EditorPropertyValue::Bool(value) => {
            *out = *value;
            true
        }
        _ => false,
    }
}

fn set_rain_glass_debug_view(value: &EditorPropertyValue, rain: &mut RainGlass2d) -> bool {
    let (EditorPropertyValue::Enum(value) | EditorPropertyValue::Text(value)) = value else {
        return false;
    };
    rain.debug_view = match value.as_str() {
        "SceneInput" => RainGlassDebugView::SceneInput,
        "BlurredScene" => RainGlassDebugView::BlurredScene,
        "RaindropMap" => RainGlassDebugView::RaindropMap,
        "DropletMap" => RainGlassDebugView::DropletMap,
        "TrailMap" => RainGlassDebugView::TrailMap,
        "DropNormals" => RainGlassDebugView::DropNormals,
        "DropMask" => RainGlassDebugView::DropMask,
        "Mist" => RainGlassDebugView::Mist,
        "Refraction" => RainGlassDebugView::Refraction,
        _ => RainGlassDebugView::Final,
    };
    true
}

fn apply_render_layer_binding(
    runtime: &Runtime,
    target: &AuthoringRuntimeBinding,
    layer_id: &str,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let service = runtime.required::<amigo_2d_composition::RenderLayer2dSceneService>()?;
    let applied = match (target, value) {
        (
            AuthoringRuntimeBinding::RenderLayerOpacity { .. },
            EditorPropertyValue::Number(value),
        ) => service.set_opacity(layer_id, value),
        (AuthoringRuntimeBinding::RenderLayerVisible { .. }, EditorPropertyValue::Bool(value)) => {
            service.set_visible(layer_id, value)
        }
        (AuthoringRuntimeBinding::RenderLayerOrder { .. }, EditorPropertyValue::Number(value)) => {
            service.set_order(layer_id, value)
        }
        (
            AuthoringRuntimeBinding::RenderLayerDepthMode { .. },
            EditorPropertyValue::Enum(value) | EditorPropertyValue::Text(value),
        ) => {
            let Some(mode) = parse_render_depth_mode(&value) else {
                return Ok(ApplyResult::Unsupported);
            };
            service.set_depth_mode(layer_id, mode)
        }
        (AuthoringRuntimeBinding::RenderLayerZDepth { .. }, EditorPropertyValue::Number(value)) => {
            service.set_z_depth(layer_id, value)
        }
        (
            AuthoringRuntimeBinding::RenderLayerDistanceM { .. },
            EditorPropertyValue::Number(value),
        ) => service.set_distance_m_with_default_space(layer_id, value),
        (
            AuthoringRuntimeBinding::RenderLayerDepthBlurScale { .. },
            EditorPropertyValue::Number(value),
        ) => service.set_depth_blur_scale(layer_id, value),
        _ => return Ok(ApplyResult::Unsupported),
    };
    if applied {
        Ok(ApplyResult::Applied)
    } else {
        Err(AmigoError::Message(format!(
            "unknown render layer `{layer_id}`"
        )))
    }
}

fn parse_render_depth_mode(value: &str) -> Option<RenderDepthMode2d> {
    match value.trim().to_ascii_lowercase().as_str() {
        "depth_map" | "depth-map" | "depthmap" => Some(RenderDepthMode2d::DepthMap),
        "distance" => Some(RenderDepthMode2d::Distance),
        "z_depth" | "z-depth" | "zdepth" => Some(RenderDepthMode2d::ZDepth),
        "infinity" => Some(RenderDepthMode2d::Infinity),
        "overlay" => Some(RenderDepthMode2d::Overlay),
        _ => None,
    }
}

fn apply_layered_image_binding(
    runtime: &Runtime,
    target: &AuthoringRuntimeBinding,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let service = runtime.required::<amigo_2d_layered_image::LayeredImageSceneService>()?;
    let applied = match (target, value) {
        (
            AuthoringRuntimeBinding::LayeredImageBaseOpacity { entity_name },
            EditorPropertyValue::Number(value),
        ) => service.set_base_opacity(entity_name, value),
        (
            AuthoringRuntimeBinding::LayeredImageLayerOpacity {
                entity_name,
                layer_id,
            },
            EditorPropertyValue::Number(value),
        ) => service.set_layer_opacity(entity_name, layer_id, value),
        (
            AuthoringRuntimeBinding::LayeredImageLayerEnabled {
                entity_name,
                layer_id,
            },
            EditorPropertyValue::Bool(value),
        ) => service.set_layer_enabled(entity_name, layer_id, value),
        _ => return Ok(ApplyResult::Unsupported),
    };
    if applied {
        Ok(ApplyResult::Applied)
    } else {
        Err(AmigoError::Message(
            "unknown layered image binding target".to_owned(),
        ))
    }
}

fn apply_mock_binding(
    state: &IngameEditorState,
    property_id: &str,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    state.set_override(property_id.to_owned(), value);
    state.set_status(format!("{property_id}: mock override [MockApplied]"));
    Ok(ApplyResult::MockApplied)
}

fn apply_particle_property(
    runtime: &Runtime,
    entity_name: &str,
    field: &str,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let service = runtime.required::<amigo_2d_particles::Particle2dSceneService>()?;
    let attempted = apply_particle_property_to_service(&service, entity_name, field, value);
    match attempted {
        Some(true) => Ok(ApplyResult::Applied),
        Some(false) => Err(AmigoError::Message(format!(
            "unknown particle emitter `{entity_name}` or invalid particle field `{field}`"
        ))),
        None => Ok(ApplyResult::Unsupported),
    }
}

fn apply_particle_property_to_service(
    service: &amigo_2d_particles::Particle2dSceneService,
    entity_name: &str,
    field: &str,
    value: EditorPropertyValue,
) -> Option<bool> {
    match (field, value) {
        ("active", EditorPropertyValue::Bool(value)) => {
            Some(service.set_active(entity_name, value))
        }
        ("spawn_rate", EditorPropertyValue::Number(value)) => {
            Some(service.set_spawn_rate(entity_name, value))
        }
        ("max_particles", EditorPropertyValue::Number(value)) => {
            Some(service.set_max_particles(entity_name, value.round().max(0.0) as usize))
        }
        ("particle_lifetime", EditorPropertyValue::Number(value)) => {
            Some(service.set_particle_lifetime(entity_name, value))
        }
        ("lifetime_jitter", EditorPropertyValue::Number(value)) => {
            Some(service.set_lifetime_jitter(entity_name, value))
        }
        ("initial_speed", EditorPropertyValue::Number(value)) => {
            Some(service.set_initial_speed(entity_name, value))
        }
        ("speed_jitter", EditorPropertyValue::Number(value)) => {
            Some(service.set_speed_jitter(entity_name, value))
        }
        ("spread_degrees", EditorPropertyValue::Number(value)) => {
            Some(service.set_spread_radians(entity_name, value.to_radians()))
        }
        ("local_direction_degrees", EditorPropertyValue::Number(value)) => {
            Some(service.set_local_direction_radians(entity_name, value.to_radians()))
        }
        ("inherit_parent_velocity", EditorPropertyValue::Number(value)) => {
            Some(service.set_inherit_parent_velocity(entity_name, value))
        }
        ("initial_size", EditorPropertyValue::Number(value)) => {
            Some(service.set_initial_size(entity_name, value))
        }
        ("final_size", EditorPropertyValue::Number(value)) => {
            Some(service.set_final_size(entity_name, value))
        }
        ("z_index", EditorPropertyValue::Number(value)) => {
            Some(service.set_z_index(entity_name, value))
        }
        ("intensity", EditorPropertyValue::Number(value)) => {
            Some(service.set_intensity(entity_name, value))
        }
        ("quality_scale", EditorPropertyValue::Number(value)) => {
            Some(service.set_quality_scale(entity_name, value))
        }
        _ => None,
    }
}
