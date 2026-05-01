#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiModelBindingKind {
    Text,
    Value,
    Visible,
    Enabled,
    Selected,
    Options,
    Color,
    Background,
    Theme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiModelBinding {
    pub path: String,
    pub state_key: String,
    pub kind: UiModelBindingKind,
    pub format: Option<String>,
}

#[derive(Debug, Default)]
pub struct UiModelBindingService {
    bindings: Mutex<Vec<UiModelBinding>>,
}
