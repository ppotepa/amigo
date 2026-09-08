use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_api::{
    AuthoringRuntimeBinding, EditorRuntimeApplyOutcome, EditorRuntimeApplyProvider,
    EditorRuntimeApplyRequest,
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
