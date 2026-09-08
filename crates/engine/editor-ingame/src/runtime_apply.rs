use amigo_2d_composition::RenderDepthMode2d;
use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_api::{EditorRuntimeApplyOutcome, EditorRuntimeApplyRequest};
use amigo_editor_authoring::AuthoringRuntimeBinding;
use amigo_runtime::Runtime;

use crate::IngameEditorRuntimeApplyProviderRegistry;
use crate::state::{EditorPropertyValue, IngameEditorState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyResult {
    Applied,
    MockApplied,
    #[allow(dead_code)]
    Readonly,
    Unsupported,
    #[allow(dead_code)]
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
            apply_runtime_property_provider(runtime, request.property_id, target, request.next)
        }
        AuthoringRuntimeBinding::ParticleEmitterProperty { entity_name, field } => {
            apply_particle_property(
                runtime,
                request.property_id,
                target,
                entity_name,
                field,
                request.next,
            )
        }
        AuthoringRuntimeBinding::PostFxFrameEnabled { .. } => {
            apply_runtime_property_provider(runtime, request.property_id, target, request.next)
        }
        AuthoringRuntimeBinding::PostFxFrameField { .. } => {
            apply_runtime_property_provider(runtime, request.property_id, target, request.next)
        }
        AuthoringRuntimeBinding::ComponentProperty { .. } => {
            apply_runtime_property_provider(runtime, request.property_id, target, request.next)
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

fn apply_runtime_property_provider(
    runtime: &Runtime,
    property_id: &str,
    target: &AuthoringRuntimeBinding,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let Some(registry) = runtime.resolve::<IngameEditorRuntimeApplyProviderRegistry>() else {
        return Ok(ApplyResult::Unsupported);
    };
    let Some(value) = editor_property_value_to_yaml(value) else {
        return Ok(ApplyResult::Unsupported);
    };
    let request = EditorRuntimeApplyRequest::RuntimeProperty {
        property_id: property_id.to_owned(),
        binding: target.clone(),
        value,
    };
    match registry.apply_first(runtime, request)? {
        Some(EditorRuntimeApplyOutcome::Applied(_)) => Ok(ApplyResult::Applied),
        Some(EditorRuntimeApplyOutcome::Ignored) | None => Ok(ApplyResult::Unsupported),
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
    property_id: &str,
    target: &AuthoringRuntimeBinding,
    _entity_name: &str,
    _field: &str,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    apply_runtime_property_provider(runtime, property_id, target, value)
}

fn editor_property_value_to_yaml(value: EditorPropertyValue) -> Option<serde_yaml::Value> {
    match value {
        EditorPropertyValue::Number(value) => serde_yaml::to_value(value).ok(),
        EditorPropertyValue::Bool(value) => Some(serde_yaml::Value::Bool(value)),
        EditorPropertyValue::Text(value)
        | EditorPropertyValue::Enum(value)
        | EditorPropertyValue::Color(value)
        | EditorPropertyValue::AssetRef(value) => Some(serde_yaml::Value::String(value)),
        EditorPropertyValue::Vec2(x, y) => serde_yaml::to_value([x, y]).ok(),
        EditorPropertyValue::Vec3(x, y, z) => serde_yaml::to_value([x, y, z]).ok(),
    }
}
