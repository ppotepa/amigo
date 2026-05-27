use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_api::{
    AuthoringRuntimeBinding, EditorRuntimeApplyOutcome, EditorRuntimeApplyProvider,
    EditorRuntimeApplyRequest,
};
use amigo_runtime::Runtime;

use crate::LayeredImageSceneService;

pub struct LayeredImageEditorRuntimeApplyProvider;

impl EditorRuntimeApplyProvider for LayeredImageEditorRuntimeApplyProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.layered-image-2d"
    }

    fn can_apply(&self, request: &EditorRuntimeApplyRequest) -> bool {
        match request {
            EditorRuntimeApplyRequest::RuntimeProperty {
                binding:
                    AuthoringRuntimeBinding::LayeredImageBaseOpacity { .. }
                    | AuthoringRuntimeBinding::LayeredImageLayerOpacity { .. }
                    | AuthoringRuntimeBinding::LayeredImageLayerEnabled { .. },
                ..
            } => true,
            EditorRuntimeApplyRequest::Command { id, .. } => matches!(
                id.as_str(),
                "editor.preview.opacity.layered-image" | "editor.preview.reveal.layered-image"
            ),
            _ => false,
        }
    }

    fn apply(
        &self,
        runtime: &Runtime,
        request: EditorRuntimeApplyRequest,
    ) -> AmigoResult<EditorRuntimeApplyOutcome> {
        let EditorRuntimeApplyRequest::RuntimeProperty { binding, value, .. } = request else {
            return apply_command(runtime, request);
        };
        let service = runtime.required::<LayeredImageSceneService>()?;
        let applied = match (binding, value) {
            (
                AuthoringRuntimeBinding::LayeredImageBaseOpacity { entity_name },
                serde_yaml::Value::Number(value),
            ) => service.set_base_opacity(&entity_name, yaml_number_to_f32(value)?),
            (
                AuthoringRuntimeBinding::LayeredImageLayerOpacity {
                    entity_name,
                    layer_id,
                },
                serde_yaml::Value::Number(value),
            ) => service.set_layer_opacity(&entity_name, &layer_id, yaml_number_to_f32(value)?),
            (
                AuthoringRuntimeBinding::LayeredImageLayerEnabled {
                    entity_name,
                    layer_id,
                },
                serde_yaml::Value::Bool(value),
            ) => service.set_layer_enabled(&entity_name, &layer_id, value),
            _ => return Ok(EditorRuntimeApplyOutcome::Ignored),
        };

        if applied {
            Ok(EditorRuntimeApplyOutcome::Applied(
                "layered image updated".to_owned(),
            ))
        } else {
            Err(AmigoError::Message(
                "unknown layered image binding target".to_owned(),
            ))
        }
    }
}

fn apply_command(
    runtime: &Runtime,
    request: EditorRuntimeApplyRequest,
) -> AmigoResult<EditorRuntimeApplyOutcome> {
    let EditorRuntimeApplyRequest::Command { id, .. } = request else {
        return Ok(EditorRuntimeApplyOutcome::Ignored);
    };
    match id.as_str() {
        "editor.preview.opacity.layered-image" => preview_opacity_report(runtime),
        "editor.preview.reveal.layered-image" => preview_reveal(runtime),
        _ => Ok(EditorRuntimeApplyOutcome::Ignored),
    }
}

fn preview_opacity_report(runtime: &Runtime) -> AmigoResult<EditorRuntimeApplyOutcome> {
    let service = runtime.required::<LayeredImageSceneService>()?;
    let Some(command) = service
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "background")
    else {
        return Ok(EditorRuntimeApplyOutcome::Ignored);
    };

    let mut lines = vec![format!(
        "layered background: base_opacity={:.3}",
        command.image.base_opacity
    )];
    for layer in [
        "club_sign",
        "club_sign_blur",
        "bar_sign",
        "bar_lanterns",
        "skyline",
    ] {
        let opacity = command
            .image
            .layer_overrides
            .iter()
            .find(|override_layer| override_layer.id == layer)
            .and_then(|override_layer| override_layer.opacity)
            .unwrap_or(1.0);
        lines.push(format!("layered background.{layer}: opacity={opacity:.3}"));
    }

    Ok(EditorRuntimeApplyOutcome::Applied(lines.join("\n")))
}

fn preview_reveal(runtime: &Runtime) -> AmigoResult<EditorRuntimeApplyOutcome> {
    let service = runtime.required::<LayeredImageSceneService>()?;
    let mut changed = Vec::new();

    if service.set_base_opacity("background", 1.0) {
        changed.push("layered background base".to_owned());
    }
    for layer in [
        "club_sign",
        "club_sign_blur",
        "bar_sign",
        "bar_sign_blur",
        "pharmacy_cross",
        "pharmacy_cross_blur",
        "bar_lanterns",
        "bar_lanterns_blur",
        "skyline",
        "skyline_blur",
        "club_entry",
        "club_entry_blur",
    ] {
        if service.set_layer_opacity("background", layer, 1.0) {
            changed.push(format!("background.{layer}"));
        }
    }

    Ok(EditorRuntimeApplyOutcome::Applied(
        changed.len().to_string(),
    ))
}

fn yaml_number_to_f32(value: serde_yaml::Number) -> AmigoResult<f32> {
    value
        .as_f64()
        .map(|value| value as f32)
        .ok_or_else(|| AmigoError::Message("invalid layered image numeric value".to_owned()))
}
