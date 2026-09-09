use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_api::{
    AuthoringRuntimeBinding, EditorRuntimeApplyOutcome, EditorRuntimeApplyProvider,
    EditorRuntimeApplyRequest,
};
use amigo_editor_authoring::{
    AuthoringNode, AuthoringSceneGraphService, AuthoringSourceValuePatch,
};
use amigo_runtime::Runtime;

use crate::NprPlaygroundState;

/// Domain-owned adapter from the generic component binding to NPR's validated
/// runtime controls. It intentionally owns no editor UI or document writer.
pub struct NprPlaygroundEditorRuntimeApplyProvider;

impl EditorRuntimeApplyProvider for NprPlaygroundEditorRuntimeApplyProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.npr-playground"
    }

    fn can_apply(&self, request: &EditorRuntimeApplyRequest) -> bool {
        matches!(
            request,
            EditorRuntimeApplyRequest::Command { id, .. } if id == "editor.save_npr_document"
        ) || matches!(
            request,
            EditorRuntimeApplyRequest::RuntimeProperty {
                binding: AuthoringRuntimeBinding::ComponentProperty { component_type, .. },
                ..
            } if component_type == "amigo.gfx.npr-playground.NprSettings"
                || component_type == "NprSettings"
        )
    }

    fn apply(
        &self,
        runtime: &Runtime,
        request: EditorRuntimeApplyRequest,
    ) -> AmigoResult<EditorRuntimeApplyOutcome> {
        if let EditorRuntimeApplyRequest::Command { id, .. } = &request {
            return match id.as_str() {
                "editor.save_npr_document" => save_npr_document(runtime),
                _ => Ok(EditorRuntimeApplyOutcome::Ignored),
            };
        }
        let EditorRuntimeApplyRequest::RuntimeProperty {
            binding:
                AuthoringRuntimeBinding::ComponentProperty {
                    component_type,
                    field,
                    ..
                },
            value,
            ..
        } = request
        else {
            return Ok(EditorRuntimeApplyOutcome::Ignored);
        };
        if component_type != "amigo.gfx.npr-playground.NprSettings" && component_type != "NprSettings" {
            return Ok(EditorRuntimeApplyOutcome::Ignored);
        }
        let state = runtime.required::<NprPlaygroundState>()?;
        match state
            .apply_editor_property(&field, value)
            .map_err(AmigoError::Message)?
        {
            true => Ok(EditorRuntimeApplyOutcome::Applied(format!(
                "NPR setting `{field}` updated"
            ))),
            false => Ok(EditorRuntimeApplyOutcome::Ignored),
        }
    }
}

fn save_npr_document(runtime: &Runtime) -> AmigoResult<EditorRuntimeApplyOutcome> {
    let authoring = runtime.required::<AuthoringSceneGraphService>()?;
    let graph = authoring.graph_for_current_scene(runtime)?;
    let Some(component) = graph
        .nodes
        .iter()
        .find_map(find_npr_component)
    else {
        return Ok(EditorRuntimeApplyOutcome::Ignored);
    };
    let state = runtime.required::<NprPlaygroundState>()?;
    let document = state.authored_scene_document().map_err(AmigoError::Message)?;
    let mut replacement = serde_yaml::to_value(document).map_err(|error| {
        AmigoError::Message(format!("cannot serialize NPR scene document: {error}"))
    })?;
    replacement
        .as_mapping_mut()
        .expect("NPR scene document serializes as a mapping")
        .insert(
            serde_yaml::Value::String("type".into()),
            serde_yaml::Value::String("amigo.gfx.npr-playground.NprSettings".into()),
        );
    authoring.apply_source_value_patch(
        runtime,
        AuthoringSourceValuePatch {
            source_file: component.source_file.clone(),
            yaml_pointer: component.yaml_pointer.clone(),
            expected: component.value.clone(),
            replacement,
        },
    )?;
    Ok(EditorRuntimeApplyOutcome::Applied(
        "NPR scene document persisted".to_owned(),
    ))
}

fn find_npr_component(node: &AuthoringNode) -> Option<&AuthoringNode> {
    matches!(
        node.semantic.component_type.as_deref(),
        Some("amigo.gfx.npr-playground.NprSettings" | "NprSettings")
    )
        .then_some(node)
        .or_else(|| node.children.iter().find_map(find_npr_component))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_structural_npr_save_command() {
        assert!(NprPlaygroundEditorRuntimeApplyProvider.can_apply(
            &EditorRuntimeApplyRequest::Command {
                id: "editor.save_npr_document".to_owned(),
                args: Vec::new(),
            }
        ));
    }

    #[test]
    fn ignores_unrelated_editor_commands() {
        assert!(!NprPlaygroundEditorRuntimeApplyProvider.can_apply(
            &EditorRuntimeApplyRequest::Command {
                id: "editor.save_other_document".to_owned(),
                args: Vec::new(),
            }
        ));
    }
}
