use std::sync::Arc;

use amigo_runtime_control::RuntimeControlService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleCompletionKind {
    Command,
    Alias,
    Argument,
    Resource,
    Property,
    Function,
    Value,
    Variable,
    Namespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleCompletionSuggestion {
    pub label: String,
    pub insert_text: String,
    pub detail: String,
    pub kind: ConsoleCompletionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleRhaiValueKind {
    Unknown,
    World,
    Scene,
    Entities,
    PostFx,
    PostFxItem,
    EntityRef,
    State,
    Session,
    Particles,
    Ui,
    Audio,
    Runtime,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleRhaiSymbol {
    pub name: String,
    pub insert_text: String,
    pub detail: String,
    pub completion_kind: ConsoleCompletionKind,
    pub value_kind: ConsoleRhaiValueKind,
}

impl ConsoleRhaiSymbol {
    pub fn variable(name: impl Into<String>, value_kind: ConsoleRhaiValueKind) -> Self {
        let name = name.into();
        Self {
            insert_text: name.clone(),
            detail: rhai_value_kind_detail(value_kind).to_owned(),
            completion_kind: ConsoleCompletionKind::Variable,
            value_kind,
            name,
        }
    }

    pub fn namespace(
        name: impl Into<String>,
        detail: impl Into<String>,
        value_kind: ConsoleRhaiValueKind,
    ) -> Self {
        let name = name.into();
        Self {
            insert_text: name.clone(),
            detail: detail.into(),
            completion_kind: ConsoleCompletionKind::Namespace,
            value_kind,
            name,
        }
    }

    pub fn function(
        name: impl Into<String>,
        insert_text: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            insert_text: insert_text.into(),
            detail: detail.into(),
            completion_kind: ConsoleCompletionKind::Function,
            value_kind: ConsoleRhaiValueKind::Function,
        }
    }
}

#[derive(Clone, Default)]
pub struct ConsoleCompletionContext {
    pub entity_names: Vec<String>,
    pub postfx_kinds: Vec<String>,
    pub postfx_indices: Vec<String>,
    pub render_layer_ids: Vec<String>,
    pub rhai_symbols: Vec<ConsoleRhaiSymbol>,
    pub runtime_control: Option<Arc<RuntimeControlService>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleCompletionEdit {
    pub input: String,
    pub cursor_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleCompletionSnapshot {
    pub input: String,
    pub cursor_index: usize,
    pub replacement_start: usize,
    pub replacement_end: usize,
    pub suggestions: Vec<ConsoleCompletionSuggestion>,
    pub selected_index: usize,
}

impl ConsoleCompletionSnapshot {
    pub fn is_active(&self) -> bool {
        !self.suggestions.is_empty()
    }

    pub fn selected(&self) -> Option<&ConsoleCompletionSuggestion> {
        self.suggestions.get(self.selected_index)
    }
}

pub(crate) fn rhai_value_kind_detail(kind: ConsoleRhaiValueKind) -> &'static str {
    match kind {
        ConsoleRhaiValueKind::Unknown => "rhai value",
        ConsoleRhaiValueKind::World => "world namespace",
        ConsoleRhaiValueKind::Scene => "scene namespace",
        ConsoleRhaiValueKind::Entities => "entities namespace",
        ConsoleRhaiValueKind::PostFx => "postfx namespace",
        ConsoleRhaiValueKind::PostFxItem => "postfx item",
        ConsoleRhaiValueKind::EntityRef => "entity reference",
        ConsoleRhaiValueKind::State => "state namespace",
        ConsoleRhaiValueKind::Session => "session namespace",
        ConsoleRhaiValueKind::Particles => "particles namespace",
        ConsoleRhaiValueKind::Ui => "ui namespace",
        ConsoleRhaiValueKind::Audio => "audio namespace",
        ConsoleRhaiValueKind::Runtime => "runtime namespace",
        ConsoleRhaiValueKind::Function => "rhai function",
    }
}
