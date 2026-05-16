#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeControlCompletionKind {
    Namespace,
    Target,
    Component,
    Property,
    Method,
    Asset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeControlCompletionEntry {
    pub label: String,
    pub insert_text: String,
    pub path: String,
    pub kind: RuntimeControlCompletionKind,
    pub detail: Option<String>,
}
