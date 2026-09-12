use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_api::{
    EditorRuntimeApplyOutcome, EditorRuntimeApplyProvider, EditorRuntimeApplyRequest,
};
use amigo_runtime::Runtime;

/// The existing editor command saves through the same authored-domain service.
pub struct NprPlaygroundEditorRuntimeApplyProvider;
impl EditorRuntimeApplyProvider for NprPlaygroundEditorRuntimeApplyProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.npr-playground"
    }
    fn can_apply(&self, request: &EditorRuntimeApplyRequest) -> bool {
        matches!(request,EditorRuntimeApplyRequest::Command{id,..} if id=="editor.save_npr_document")
    }
    fn apply(
        &self,
        runtime: &Runtime,
        request: EditorRuntimeApplyRequest,
    ) -> AmigoResult<EditorRuntimeApplyOutcome> {
        if !self.can_apply(&request) {
            return Ok(EditorRuntimeApplyOutcome::Ignored);
        }
        let service = runtime.required::<crate::playground::NprPlaygroundService>()?;
        service
            .dispatch_intent(
                0,
                service.domain_snapshot().revision,
                "editor.save".into(),
                crate::playground::NprPlaygroundIntent::SaveAll,
            )
            .map_err(|e| AmigoError::Message(e.message))?;
        Ok(EditorRuntimeApplyOutcome::Applied(
            "NPR scene profile persisted".into(),
        ))
    }
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
