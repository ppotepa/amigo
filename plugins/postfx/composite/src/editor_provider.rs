use amigo_core::AmigoResult;
use amigo_editor_api::{
    AuthoringRuntimeBinding, EditorRuntimeApplyOutcome, EditorRuntimeApplyProvider,
    EditorRuntimeApplyRequest,
};
use amigo_runtime::Runtime;

use crate::{PostFx2d, PostFx2dService, RainGlass2d, RainGlassDebugView};

pub struct CompositeEditorRuntimeApplyProvider;

impl EditorRuntimeApplyProvider for CompositeEditorRuntimeApplyProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.postfx.composite"
    }

    fn can_apply(&self, request: &EditorRuntimeApplyRequest) -> bool {
        matches!(
            request,
            EditorRuntimeApplyRequest::RuntimeProperty {
                binding:
                    AuthoringRuntimeBinding::PostFxFrameEnabled { .. }
                    | AuthoringRuntimeBinding::PostFxFrameField { .. },
                ..
            }
        )
    }

    fn apply(
        &self,
        runtime: &Runtime,
        request: EditorRuntimeApplyRequest,
    ) -> AmigoResult<EditorRuntimeApplyOutcome> {
        let EditorRuntimeApplyRequest::RuntimeProperty { binding, value, .. } = request else {
            return Ok(EditorRuntimeApplyOutcome::Ignored);
        };
        match binding {
            AuthoringRuntimeBinding::PostFxFrameEnabled { index } => {
                apply_postfx_frame_enabled(runtime, index, value)
            }
            AuthoringRuntimeBinding::PostFxFrameField { index, field } => {
                apply_postfx_frame_field(runtime, index, &field, value)
            }
            _ => Ok(EditorRuntimeApplyOutcome::Ignored),
        }
    }
}

fn apply_postfx_frame_enabled(
    runtime: &Runtime,
    index: usize,
    value: serde_yaml::Value,
) -> AmigoResult<EditorRuntimeApplyOutcome> {
    let serde_yaml::Value::Bool(value) = value else {
        return Ok(EditorRuntimeApplyOutcome::Ignored);
    };
    let service = runtime.required::<PostFx2dService>()?;
    if service.set_frame_effect_enabled(index, value) {
        Ok(EditorRuntimeApplyOutcome::Applied(format!(
            "postfx frame effect {index} updated"
        )))
    } else {
        Ok(EditorRuntimeApplyOutcome::Ignored)
    }
}

fn apply_postfx_frame_field(
    runtime: &Runtime,
    index: usize,
    field: &str,
    value: serde_yaml::Value,
) -> AmigoResult<EditorRuntimeApplyOutcome> {
    let service = runtime.required::<PostFx2dService>()?;
    let applied = service.update_frame_effect(index, |effect| match effect {
        PostFx2d::RainGlass(mut rain) => {
            if !apply_rain_glass_field(&mut rain, field, &value) {
                return None;
            }
            Some(PostFx2d::RainGlass(rain.normalized()))
        }
        other => Some(other),
    });
    if applied {
        Ok(EditorRuntimeApplyOutcome::Applied(format!(
            "postfx field `{field}` updated"
        )))
    } else {
        Ok(EditorRuntimeApplyOutcome::Ignored)
    }
}

fn apply_rain_glass_field(rain: &mut RainGlass2d, field: &str, value: &serde_yaml::Value) -> bool {
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

fn set_f32(value: &serde_yaml::Value, out: &mut f32) -> bool {
    let serde_yaml::Value::Number(value) = value else {
        return false;
    };
    let Some(value) = value.as_f64() else {
        return false;
    };
    *out = value as f32;
    true
}

fn set_bool(value: &serde_yaml::Value, out: &mut bool) -> bool {
    let serde_yaml::Value::Bool(value) = value else {
        return false;
    };
    *out = *value;
    true
}

fn set_rain_glass_debug_view(value: &serde_yaml::Value, rain: &mut RainGlass2d) -> bool {
    let serde_yaml::Value::String(value) = value else {
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
