use amigo_core::AmigoResult;
use amigo_runtime::Runtime;

#[derive(Debug, Clone)]
pub enum EditorRuntimeApplyRequest {
    SetProperty {
        node_id: String,
        property_id: String,
        value: serde_yaml::Value,
    },
    Command {
        id: String,
        args: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum EditorRuntimeApplyOutcome {
    Applied(String),
    Ignored,
}

pub trait EditorRuntimeApplyProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn can_apply(&self, request: &EditorRuntimeApplyRequest) -> bool;

    fn apply(
        &self,
        runtime: &Runtime,
        request: EditorRuntimeApplyRequest,
    ) -> AmigoResult<EditorRuntimeApplyOutcome>;
}
