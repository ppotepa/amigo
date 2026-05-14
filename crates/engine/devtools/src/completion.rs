use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::ConsoleCommandDescriptor;

const MAX_COMPLETION_SUGGESTIONS: usize = 8;

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsoleCompletionContext {
    pub entity_names: Vec<String>,
    pub postfx_kinds: Vec<String>,
    pub postfx_indices: Vec<String>,
    pub rhai_symbols: Vec<ConsoleRhaiSymbol>,
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

#[derive(Debug, Default)]
struct ConsoleCompletionInner {
    snapshot: Option<ConsoleCompletionSnapshot>,
}

#[derive(Debug, Default)]
pub struct ConsoleCompletionState {
    inner: Mutex<ConsoleCompletionInner>,
}

impl ConsoleCompletionState {
    pub fn snapshot(&self) -> Option<ConsoleCompletionSnapshot> {
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot
            .clone()
    }

    pub fn clear(&self) {
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot = None;
    }

    pub fn refresh(
        &self,
        input: &str,
        cursor_index: usize,
        descriptors: &[ConsoleCommandDescriptor],
        context: &ConsoleCompletionContext,
    ) {
        let snapshot =
            compute_console_completion_from_descriptors(input, cursor_index, descriptors, context);
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot = snapshot.filter(ConsoleCompletionSnapshot::is_active);
    }

    pub fn select_next(&self) -> bool {
        self.select_delta(1)
    }

    pub fn select_previous(&self) -> bool {
        self.select_delta(-1)
    }

    pub fn accept_tab(&self, input: &str, cursor_index: usize) -> Option<ConsoleCompletionEdit> {
        let snapshot = self.snapshot()?;
        if snapshot.input != input || snapshot.cursor_index != cursor_index {
            return None;
        }
        accept_completion_tab(input, &snapshot)
    }

    fn select_delta(&self, delta: isize) -> bool {
        let mut inner = self
            .inner
            .lock()
            .expect("console completion mutex should not be poisoned");
        let Some(snapshot) = inner.snapshot.as_mut() else {
            return false;
        };
        if snapshot.suggestions.is_empty() {
            return false;
        }

        let len = snapshot.suggestions.len() as isize;
        let next = (snapshot.selected_index as isize + delta).rem_euclid(len);
        snapshot.selected_index = next as usize;
        true
    }
}

pub fn compute_console_completion_from_descriptors(
    input: &str,
    cursor_index: usize,
    descriptors: &[ConsoleCommandDescriptor],
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    let cursor_index = clamp_to_char_boundary(input, cursor_index.min(input.len()));

    if let Some(snapshot) = complete_runtime_context(input, cursor_index, context) {
        return Some(snapshot);
    }

    if let Some(snapshot) = complete_rhai_property(input, cursor_index, context) {
        return Some(snapshot);
    }

    let token = active_token(input, cursor_index);
    if token.command_token {
        return complete_initial_token(
            input,
            cursor_index,
            token.start,
            token.end,
            token.value,
            descriptors,
            context,
        );
    }

    if let Some(snapshot) = complete_command_segment(
        input,
        cursor_index,
        token.start,
        token.end,
        token.value,
        descriptors,
    ) {
        return Some(snapshot);
    }

    if let Some(snapshot) = complete_argument(
        input,
        cursor_index,
        token.start,
        token.end,
        token.value,
        descriptors,
    ) {
        return Some(snapshot);
    }

    complete_rhai_symbol(
        input,
        cursor_index,
        token.start,
        token.end,
        token.value,
        context,
    )
}

pub fn accept_completion_tab(
    input: &str,
    snapshot: &ConsoleCompletionSnapshot,
) -> Option<ConsoleCompletionEdit> {
    if snapshot.suggestions.is_empty() {
        return None;
    }

    let replacement = current_replacement(input, snapshot);
    if snapshot.suggestions.len() == 1 {
        return Some(apply_suggestion(input, snapshot, &snapshot.suggestions[0]));
    }

    if let Some(prefix) = common_insert_prefix(&snapshot.suggestions) {
        if prefix.len() > replacement.len() && prefix.starts_with(replacement) {
            return Some(apply_insert_text(input, snapshot, &prefix, false));
        }
    }

    snapshot
        .selected()
        .map(|suggestion| apply_suggestion(input, snapshot, suggestion))
}

#[derive(Debug, Clone, Copy)]
struct ActiveToken<'a> {
    start: usize,
    end: usize,
    value: &'a str,
    command_token: bool,
}

fn active_token(input: &str, cursor_index: usize) -> ActiveToken<'_> {
    let cursor_index = clamp_to_char_boundary(input, cursor_index.min(input.len()));

    let start = input[..cursor_index]
        .char_indices()
        .rev()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index + ch.len_utf8()))
        .unwrap_or(0);

    let end = input[cursor_index..]
        .char_indices()
        .find_map(|(offset, ch)| ch.is_whitespace().then_some(cursor_index + offset))
        .unwrap_or(input.len());

    let command_token = !input[..start].contains(char::is_whitespace);

    ActiveToken {
        start,
        end,
        value: &input[start..cursor_index],
        command_token,
    }
}

fn clamp_to_char_boundary(input: &str, mut index: usize) -> usize {
    index = index.min(input.len());
    while index > 0 && !input.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn complete_initial_token(
    input: &str,
    cursor_index: usize,
    start: usize,
    end: usize,
    prefix: &str,
    descriptors: &[ConsoleCommandDescriptor],
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    if prefix.trim().is_empty() {
        return None;
    }

    let mut suggestions = command_name_suggestions(prefix, descriptors);
    suggestions.extend(rhai_symbol_suggestions(
        input,
        cursor_index,
        prefix,
        context,
    ));
    sort_and_limit_suggestions(&mut suggestions);
    if suggestions.is_empty() {
        return None;
    }

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        cursor_index,
        replacement_start: start,
        replacement_end: end,
        suggestions,
        selected_index: 0,
    })
}

fn command_name_suggestions(
    prefix: &str,
    descriptors: &[ConsoleCommandDescriptor],
) -> Vec<ConsoleCompletionSuggestion> {
    let mut suggestions = Vec::new();
    for descriptor in descriptors {
        if descriptor.name.starts_with(prefix) {
            suggestions.push(ConsoleCompletionSuggestion {
                label: descriptor.name.to_owned(),
                insert_text: format!("{} ", descriptor.name),
                detail: descriptor.help.to_owned(),
                kind: ConsoleCompletionKind::Command,
            });
        }
        for alias in descriptor.aliases {
            if alias.starts_with(prefix) {
                suggestions.push(ConsoleCompletionSuggestion {
                    label: (*alias).to_owned(),
                    insert_text: format!("{alias} "),
                    detail: format!("alias for {}", descriptor.name),
                    kind: ConsoleCompletionKind::Alias,
                });
            }
        }
    }
    suggestions
}

fn complete_command_segment(
    input: &str,
    cursor_index: usize,
    start: usize,
    end: usize,
    prefix: &str,
    descriptors: &[ConsoleCommandDescriptor],
) -> Option<ConsoleCompletionSnapshot> {
    let previous_tokens = input[..start].split_whitespace().collect::<Vec<_>>();
    if previous_tokens.is_empty() {
        return None;
    }

    let command_prefix = previous_tokens.join(".");
    let nested_prefix = format!("{command_prefix}.");
    if descriptors
        .iter()
        .any(|descriptor| descriptor.name == command_prefix)
    {
        return None;
    }

    let mut suggestions = descriptors
        .iter()
        .filter_map(|descriptor| descriptor.name.strip_prefix(&nested_prefix))
        .filter_map(|remaining| remaining.split('.').next())
        .filter(|segment| segment.starts_with(prefix))
        .map(|segment| ConsoleCompletionSuggestion {
            label: segment.to_owned(),
            insert_text: format!("{segment} "),
            detail: format!("{command_prefix} subcommand"),
            kind: ConsoleCompletionKind::Command,
        })
        .collect::<Vec<_>>();

    suggestions.sort_by(|a, b| a.label.cmp(&b.label));
    suggestions.dedup_by(|a, b| a.label == b.label);
    suggestions.truncate(MAX_COMPLETION_SUGGESTIONS);
    if suggestions.is_empty() {
        return None;
    }

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        cursor_index,
        replacement_start: start,
        replacement_end: end,
        suggestions,
        selected_index: 0,
    })
}

fn complete_argument(
    input: &str,
    cursor_index: usize,
    start: usize,
    end: usize,
    prefix: &str,
    descriptors: &[ConsoleCommandDescriptor],
) -> Option<ConsoleCompletionSnapshot> {
    let command_name = input.split_whitespace().next()?;
    let descriptor = descriptors.iter().find(|descriptor| {
        descriptor.name == command_name || descriptor.aliases.contains(&command_name)
    })?;

    let arg_index = input[..start].split_whitespace().skip(1).count();
    let values = usage_enum_values(descriptor.usage, arg_index)?;
    let mut suggestions = values
        .into_iter()
        .filter(|value| value.starts_with(prefix))
        .map(|value| ConsoleCompletionSuggestion {
            label: value.to_owned(),
            insert_text: format!("{value} "),
            detail: descriptor.usage.to_owned(),
            kind: ConsoleCompletionKind::Argument,
        })
        .collect::<Vec<_>>();

    suggestions.sort_by(|a, b| a.label.cmp(&b.label));
    suggestions.truncate(MAX_COMPLETION_SUGGESTIONS);
    if suggestions.is_empty() {
        return None;
    }

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        cursor_index,
        replacement_start: start,
        replacement_end: end,
        suggestions,
        selected_index: 0,
    })
}

fn complete_runtime_context(
    input: &str,
    cursor_index: usize,
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    if let Some(snapshot) = complete_quoted_entity_name(input, cursor_index, context) {
        return Some(snapshot);
    }

    let token = active_token(input, cursor_index);
    let before = &input[..token.start];

    if before.ends_with("scene.entities inspect ")
        || before.ends_with("scene.entities remove ")
        || before.ends_with("entity ")
    {
        return complete_values(
            input,
            cursor_index,
            token.start,
            token.end,
            token.value,
            &context.entity_names,
            ConsoleCompletionKind::Value,
            "scene entity",
            true,
        );
    }

    if before.ends_with("postfx.items add ") {
        return complete_values(
            input,
            cursor_index,
            token.start,
            token.end,
            token.value,
            &context.postfx_kinds,
            ConsoleCompletionKind::Value,
            "postfx kind",
            true,
        );
    }

    if before.ends_with("postfx.items inspect ") || before.ends_with("postfx.items remove ") {
        return complete_values(
            input,
            cursor_index,
            token.start,
            token.end,
            token.value,
            &context.postfx_indices,
            ConsoleCompletionKind::Value,
            "postfx item index",
            true,
        );
    }

    None
}

fn complete_values(
    input: &str,
    cursor_index: usize,
    start: usize,
    end: usize,
    prefix: &str,
    values: &[String],
    kind: ConsoleCompletionKind,
    detail: &str,
    trailing_space: bool,
) -> Option<ConsoleCompletionSnapshot> {
    let mut suggestions = values
        .iter()
        .filter(|value| value.starts_with(prefix))
        .map(|value| ConsoleCompletionSuggestion {
            label: value.clone(),
            insert_text: if trailing_space {
                format!("{value} ")
            } else {
                value.clone()
            },
            detail: detail.to_owned(),
            kind,
        })
        .collect::<Vec<_>>();

    suggestions.sort_by(|a, b| a.label.cmp(&b.label));
    suggestions.dedup_by(|a, b| a.label == b.label);
    suggestions.truncate(MAX_COMPLETION_SUGGESTIONS);
    if suggestions.is_empty() {
        return None;
    }

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        cursor_index,
        replacement_start: start,
        replacement_end: end,
        suggestions,
        selected_index: 0,
    })
}

fn sort_and_limit_suggestions(suggestions: &mut Vec<ConsoleCompletionSuggestion>) {
    suggestions.sort_by(|a, b| {
        a.label
            .cmp(&b.label)
            .then_with(|| a.insert_text.cmp(&b.insert_text))
    });
    suggestions.dedup_by(|a, b| a.label == b.label && a.insert_text == b.insert_text);
    suggestions.truncate(MAX_COMPLETION_SUGGESTIONS);
}

fn complete_rhai_symbol(
    input: &str,
    cursor_index: usize,
    start: usize,
    end: usize,
    prefix: &str,
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    if prefix.trim().is_empty() || !is_rhai_identifier_prefix(prefix) {
        return None;
    }

    let mut suggestions = rhai_symbol_suggestions(input, cursor_index, prefix, context);
    sort_and_limit_suggestions(&mut suggestions);
    if suggestions.is_empty() {
        return None;
    }

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        cursor_index,
        replacement_start: start,
        replacement_end: end,
        suggestions,
        selected_index: 0,
    })
}

fn rhai_symbol_suggestions(
    input: &str,
    cursor_index: usize,
    prefix: &str,
    context: &ConsoleCompletionContext,
) -> Vec<ConsoleCompletionSuggestion> {
    if prefix.trim().is_empty() || !is_rhai_identifier_prefix(prefix) {
        return Vec::new();
    }

    rhai_symbols_for_context(input, cursor_index, context)
        .into_iter()
        .filter(|symbol| {
            symbol.name.starts_with(prefix)
                && !(symbol.name == prefix && symbol.insert_text == prefix)
        })
        .map(|symbol| ConsoleCompletionSuggestion {
            label: symbol.name,
            insert_text: symbol.insert_text,
            detail: symbol.detail,
            kind: symbol.completion_kind,
        })
        .collect()
}

fn rhai_symbols_for_context(
    input: &str,
    cursor_index: usize,
    context: &ConsoleCompletionContext,
) -> Vec<ConsoleRhaiSymbol> {
    let mut by_name = BTreeMap::new();
    for symbol in builtin_rhai_symbols()
        .into_iter()
        .chain(context.rhai_symbols.iter().cloned())
        .chain(collect_console_rhai_symbols_from_source(&input[..cursor_index]).into_iter())
    {
        by_name.insert(symbol.name.clone(), symbol);
    }
    by_name.into_iter().map(|(_, symbol)| symbol).collect()
}

fn builtin_rhai_symbols() -> Vec<ConsoleRhaiSymbol> {
    vec![
        ConsoleRhaiSymbol::namespace("world", "Rhai root API", ConsoleRhaiValueKind::World),
        ConsoleRhaiSymbol::namespace("scene", "Rhai scene API", ConsoleRhaiValueKind::Scene),
        ConsoleRhaiSymbol::namespace(
            "entities",
            "Rhai entity API",
            ConsoleRhaiValueKind::Entities,
        ),
        ConsoleRhaiSymbol::namespace("postfx", "Rhai post-fx API", ConsoleRhaiValueKind::PostFx),
        ConsoleRhaiSymbol::namespace("state", "Rhai scene state API", ConsoleRhaiValueKind::State),
        ConsoleRhaiSymbol::namespace(
            "session",
            "Rhai session state API",
            ConsoleRhaiValueKind::Session,
        ),
        ConsoleRhaiSymbol::namespace(
            "particles",
            "Rhai particles API",
            ConsoleRhaiValueKind::Particles,
        ),
        ConsoleRhaiSymbol::namespace("ui", "Rhai UI API", ConsoleRhaiValueKind::Ui),
        ConsoleRhaiSymbol::namespace("audio", "Rhai audio API", ConsoleRhaiValueKind::Audio),
        ConsoleRhaiSymbol::namespace("runtime", "Rhai runtime API", ConsoleRhaiValueKind::Runtime),
        ConsoleRhaiSymbol::function(
            "get_entity",
            "get_entity(\"",
            "Rhai shortcut: entity by name",
        ),
        ConsoleRhaiSymbol::function("entity", "entity(\"", "Rhai shortcut: entity by name"),
        ConsoleRhaiSymbol::function(
            "list_entities",
            "list_entities()",
            "Rhai shortcut: list entity names",
        ),
        ConsoleRhaiSymbol::function(
            "list_postfx_items",
            "list_postfx_items()",
            "Rhai shortcut: list post-fx items",
        ),
    ]
}

pub fn collect_console_rhai_symbols_from_source(source: &str) -> Vec<ConsoleRhaiSymbol> {
    source
        .split(|ch| ch == ';' || ch == '\n')
        .filter_map(declared_rhai_symbol_from_statement)
        .collect()
}

fn declared_rhai_symbol_from_statement(statement: &str) -> Option<ConsoleRhaiSymbol> {
    let statement = statement.trim_start();
    let after_keyword = statement
        .strip_prefix("let ")
        .or_else(|| statement.strip_prefix("const "))?;
    let after_keyword = after_keyword.trim_start();
    let name_end = rhai_identifier_end(after_keyword)?;
    let name = &after_keyword[..name_end];
    if !is_rhai_identifier(name) {
        return None;
    }

    let value_kind = after_keyword[name_end..]
        .trim_start()
        .strip_prefix('=')
        .map(infer_rhai_value_kind_from_expression)
        .unwrap_or(ConsoleRhaiValueKind::Unknown);

    Some(ConsoleRhaiSymbol::variable(name, value_kind))
}

fn infer_rhai_value_kind_from_expression(expression: &str) -> ConsoleRhaiValueKind {
    let expression = expression.trim_start();
    if expression.starts_with("get_entity(")
        || expression.starts_with("entity(")
        || expression.starts_with("entities.named(")
    {
        ConsoleRhaiValueKind::EntityRef
    } else if expression.starts_with("postfx.item(") || expression.starts_with("postfx.items[") {
        ConsoleRhaiValueKind::PostFxItem
    } else {
        match trim_expression_tail(expression) {
            "world" => ConsoleRhaiValueKind::World,
            "scene" => ConsoleRhaiValueKind::Scene,
            "entities" => ConsoleRhaiValueKind::Entities,
            "postfx" => ConsoleRhaiValueKind::PostFx,
            "state" => ConsoleRhaiValueKind::State,
            "session" => ConsoleRhaiValueKind::Session,
            "particles" => ConsoleRhaiValueKind::Particles,
            "ui" => ConsoleRhaiValueKind::Ui,
            "audio" => ConsoleRhaiValueKind::Audio,
            "runtime" => ConsoleRhaiValueKind::Runtime,
            _ => ConsoleRhaiValueKind::Unknown,
        }
    }
}

fn trim_expression_tail(expression: &str) -> &str {
    expression
        .trim()
        .trim_end_matches(';')
        .trim_end_matches(|ch: char| ch.is_whitespace())
}

fn rhai_identifier_end(value: &str) -> Option<usize> {
    let mut end = 0;
    for (index, ch) in value.char_indices() {
        if index == 0 {
            if !is_rhai_identifier_start(ch) {
                return None;
            }
            end = ch.len_utf8();
            continue;
        }
        if !is_rhai_identifier_continue(ch) {
            break;
        }
        end = index + ch.len_utf8();
    }
    (end > 0).then_some(end)
}

fn is_rhai_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_rhai_identifier_start(first) && chars.all(is_rhai_identifier_continue)
}

fn is_rhai_identifier_prefix(value: &str) -> bool {
    value.chars().all(is_rhai_identifier_continue)
}

fn is_rhai_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_rhai_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn rhai_value_kind_detail(kind: ConsoleRhaiValueKind) -> &'static str {
    match kind {
        ConsoleRhaiValueKind::Unknown => "Rhai variable",
        ConsoleRhaiValueKind::World => "Rhai world root",
        ConsoleRhaiValueKind::Scene => "Rhai scene API",
        ConsoleRhaiValueKind::Entities => "Rhai entity API",
        ConsoleRhaiValueKind::PostFx => "Rhai post-fx API",
        ConsoleRhaiValueKind::PostFxItem => "Rhai post-fx item",
        ConsoleRhaiValueKind::EntityRef => "Rhai entity ref",
        ConsoleRhaiValueKind::State => "Rhai scene state API",
        ConsoleRhaiValueKind::Session => "Rhai session state API",
        ConsoleRhaiValueKind::Particles => "Rhai particles API",
        ConsoleRhaiValueKind::Ui => "Rhai UI API",
        ConsoleRhaiValueKind::Audio => "Rhai audio API",
        ConsoleRhaiValueKind::Runtime => "Rhai runtime API",
        ConsoleRhaiValueKind::Function => "Rhai function",
    }
}

fn complete_quoted_entity_name(
    input: &str,
    cursor_index: usize,
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    let before = &input[..cursor_index];

    let start = if let Some(index) = before.rfind("get_entity(\"") {
        index + "get_entity(\"".len()
    } else if let Some(index) = before.rfind("entity(\"") {
        index + "entity(\"".len()
    } else if let Some(index) = before.rfind("scene.entities[\"") {
        index + "scene.entities[\"".len()
    } else {
        return None;
    };

    if start > cursor_index {
        return None;
    }

    let prefix = &input[start..cursor_index];

    complete_values(
        input,
        cursor_index,
        start,
        cursor_index,
        prefix,
        &context.entity_names,
        ConsoleCompletionKind::Value,
        "scene entity",
        false,
    )
}

fn complete_rhai_property(
    input: &str,
    cursor_index: usize,
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    let token = active_token(input, cursor_index);
    let token_value = &input[token.start..cursor_index];
    let dot_offset = token_value.rfind('.')?;

    let expression = token_value[..dot_offset].trim();
    let property_prefix = &token_value[dot_offset + 1..];
    if !property_prefix
        .chars()
        .all(|ch| is_rhai_identifier_continue(ch) || ch == '(')
    {
        return None;
    }

    let symbols = rhai_symbols_for_context(input, cursor_index, context);
    let value_kind = infer_rhai_value_kind_for_property_expression(expression, &symbols)?;
    let properties = rhai_properties_for_value_kind(value_kind)?;
    let replacement_start = token.start + dot_offset + 1;

    complete_values(
        input,
        cursor_index,
        replacement_start,
        token.end,
        property_prefix,
        &properties,
        ConsoleCompletionKind::Property,
        rhai_value_kind_detail(value_kind),
        false,
    )
}

fn infer_rhai_value_kind_for_property_expression(
    expression: &str,
    symbols: &[ConsoleRhaiSymbol],
) -> Option<ConsoleRhaiValueKind> {
    if expression.starts_with("postfx.item(") || expression.starts_with("postfx.items[") {
        return Some(ConsoleRhaiValueKind::PostFxItem);
    }
    if expression.starts_with("get_entity(")
        || expression.starts_with("entity(")
        || expression.starts_with("scene.entities[")
        || expression.starts_with("entities.named(")
    {
        return Some(ConsoleRhaiValueKind::EntityRef);
    }

    let exact_kind = match expression {
        "world" => Some(ConsoleRhaiValueKind::World),
        "scene" => Some(ConsoleRhaiValueKind::Scene),
        "entities" => Some(ConsoleRhaiValueKind::Entities),
        "postfx" => Some(ConsoleRhaiValueKind::PostFx),
        "state" => Some(ConsoleRhaiValueKind::State),
        "session" => Some(ConsoleRhaiValueKind::Session),
        "particles" => Some(ConsoleRhaiValueKind::Particles),
        "ui" => Some(ConsoleRhaiValueKind::Ui),
        "audio" => Some(ConsoleRhaiValueKind::Audio),
        "runtime" => Some(ConsoleRhaiValueKind::Runtime),
        _ => None,
    };
    if exact_kind.is_some() {
        return exact_kind;
    }

    symbols
        .iter()
        .find(|symbol| symbol.name == expression)
        .map(|symbol| symbol.value_kind)
}

fn rhai_properties_for_value_kind(kind: ConsoleRhaiValueKind) -> Option<Vec<String>> {
    let properties: &[&str] = match kind {
        ConsoleRhaiValueKind::World => &[
            "scene",
            "entities",
            "postfx",
            "state",
            "session",
            "particles",
            "ui",
            "audio",
            "runtime",
        ],
        ConsoleRhaiValueKind::Scene => &[
            "current_id()",
            "available()",
            "has(",
            "select(",
            "reload()",
            "activate_set(",
        ],
        ConsoleRhaiValueKind::Entities => &[
            "named(",
            "create(",
            "exists(",
            "count()",
            "names()",
            "distance(",
            "hide(",
            "show(",
            "enable(",
            "disable(",
            "is_visible(",
            "is_enabled(",
            "collision_enabled(",
            "by_tag(",
            "by_group(",
            "property(",
            "set_property_int(",
            "set_property_float(",
            "set_property_bool(",
            "set_property_string(",
        ],
        ConsoleRhaiValueKind::PostFx => &["count()", "list()", "item("],
        ConsoleRhaiValueKind::PostFxItem => &["exists", "index", "name", "active", "enabled"],
        ConsoleRhaiValueKind::EntityRef => &[
            "name",
            "exists",
            "opacity",
            "visible",
            "enabled",
            "collision_enabled",
        ],
        ConsoleRhaiValueKind::State => &[
            "set_int(",
            "set_float(",
            "set_bool(",
            "set_string(",
            "get_int(",
            "get_float(",
            "get_bool(",
            "get_string(",
            "add_int(",
            "add_float(",
            "add_bool(",
            "add_string(",
            "reset_scene()",
        ],
        ConsoleRhaiValueKind::Session => &[
            "set_int(",
            "set_float(",
            "set_bool(",
            "set_string(",
            "get_int(",
            "get_float(",
            "get_bool(",
            "get_string(",
            "add_int(",
            "add_float(",
            "add_bool(",
            "add_string(",
        ],
        ConsoleRhaiValueKind::Particles => &[
            "start(",
            "stop(",
            "emit(",
            "burst(",
            "burst_at(",
            "preset_ids()",
            "apply_preset(",
            "export_yaml(",
        ],
        ConsoleRhaiValueKind::Ui => &[
            "set_text(",
            "set_many(",
            "set_value(",
            "set_selected(",
            "set_options(",
            "set_color(",
            "set_background(",
            "show(",
            "hide(",
            "enable(",
            "disable(",
            "set_theme(",
            "theme()",
        ],
        ConsoleRhaiValueKind::Audio => &["play(", "cue(", "preload(", "play_asset(", "stop_all()"],
        ConsoleRhaiValueKind::Runtime => &["backend()", "mod_id()", "scene_id()", "diagnostics()"],
        ConsoleRhaiValueKind::Unknown | ConsoleRhaiValueKind::Function => return None,
    };

    Some(
        properties
            .iter()
            .map(|property| (*property).to_owned())
            .collect(),
    )
}

fn usage_enum_values(usage: &str, arg_index: usize) -> Option<Vec<&str>> {
    let token = usage.split_whitespace().nth(1 + arg_index)?;
    let token = token
        .trim_matches('[')
        .trim_matches(']')
        .trim_matches('<')
        .trim_matches('>');
    token.contains('|').then(|| token.split('|').collect())
}

fn apply_suggestion(
    input: &str,
    snapshot: &ConsoleCompletionSnapshot,
    suggestion: &ConsoleCompletionSuggestion,
) -> ConsoleCompletionEdit {
    apply_insert_text(input, snapshot, &suggestion.insert_text, true)
}

fn apply_insert_text(
    input: &str,
    snapshot: &ConsoleCompletionSnapshot,
    insert_text: &str,
    preserve_spacing: bool,
) -> ConsoleCompletionEdit {
    let mut next = String::new();
    next.push_str(&input[..snapshot.replacement_start]);

    let insert_start = next.len();
    next.push_str(insert_text);

    if !preserve_spacing && !insert_text.ends_with(' ') {
        next.push(' ');
    }

    let cursor_index = insert_start
        + insert_text.len()
        + usize::from(!preserve_spacing && !insert_text.ends_with(' '));

    next.push_str(&input[snapshot.replacement_end..]);

    ConsoleCompletionEdit {
        input: next,
        cursor_index,
    }
}

fn current_replacement<'a>(input: &'a str, snapshot: &ConsoleCompletionSnapshot) -> &'a str {
    &input[snapshot.replacement_start..snapshot.cursor_index]
}

fn common_insert_prefix(suggestions: &[ConsoleCompletionSuggestion]) -> Option<String> {
    let first = suggestions.first()?.insert_text.as_str();
    let mut prefix_len = first.len();
    for suggestion in suggestions.iter().skip(1) {
        prefix_len = common_prefix_len(&first[..prefix_len], &suggestion.insert_text);
    }
    Some(first[..prefix_len].to_owned())
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.char_indices()
        .zip(right.char_indices())
        .take_while(|((_, a), (_, b))| a == b)
        .map(|((index, ch), _)| index + ch.len_utf8())
        .last()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        ConsoleCommandDescriptor, ConsoleCompletionContext, ConsoleRhaiSymbol,
        ConsoleRhaiValueKind, collect_console_rhai_symbols_from_source,
        compute_console_completion_from_descriptors,
    };

    #[test]
    fn completes_command_prefix() {
        let descriptors = [
            ConsoleCommandDescriptor {
                name: "debug.fps",
                aliases: &[],
                category: "debug",
                help: "Show FPS.",
                usage: "debug.fps on|off|toggle",
                examples: &["debug.fps on"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "debug.fps_graph",
                aliases: &[],
                category: "debug",
                help: "Show FPS graph.",
                usage: "debug.fps_graph on|off|toggle",
                examples: &["debug.fps_graph on"],
                dev_only: true,
            },
        ];

        let completion = compute_console_completion_from_descriptors(
            "debug.fp",
            "debug.fp".len(),
            &descriptors,
            &ConsoleCompletionContext::default(),
        )
        .unwrap();
        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "debug.fps")
        );
        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "debug.fps_graph")
        );
    }

    #[test]
    fn completes_enum_argument_from_usage() {
        let descriptors = [ConsoleCommandDescriptor {
            name: "debug.fps",
            aliases: &[],
            category: "debug",
            help: "Show FPS.",
            usage: "debug.fps on|off|toggle",
            examples: &["debug.fps on"],
            dev_only: true,
        }];

        let completion = compute_console_completion_from_descriptors(
            "debug.fps o",
            "debug.fps o".len(),
            &descriptors,
            &ConsoleCompletionContext::default(),
        )
        .unwrap();
        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["off", "on"]
        );
    }

    #[test]
    fn state_tracks_snapshot() {
        let descriptors = [ConsoleCommandDescriptor {
            name: "debug.fps",
            aliases: &[],
            category: "debug",
            help: "Show FPS.",
            usage: "debug.fps on|off|toggle",
            examples: &["debug.fps on"],
            dev_only: true,
        }];
        let state = super::ConsoleCompletionState::default();

        state.refresh(
            "debug.fp",
            "debug.fp".len(),
            &descriptors,
            &ConsoleCompletionContext::default(),
        );
        assert!(state.snapshot().is_some());
        assert!(state.select_next());
        assert_eq!(
            state
                .accept_tab("debug.fp", "debug.fp".len())
                .map(|edit| edit.input)
                .as_deref(),
            Some("debug.fps ")
        );
        state.clear();
        assert!(state.snapshot().is_none());
    }

    #[test]
    fn completes_token_at_cursor_not_only_line_end() {
        let descriptors = [ConsoleCommandDescriptor {
            name: "postfx.items",
            aliases: &[],
            category: "render",
            help: "Postfx items.",
            usage: "postfx.items <list|count|add|clear|inspect> [args...]",
            examples: &["postfx.items list"],
            dev_only: true,
        }];

        let input = "postfx.items ad blur";
        let cursor = "postfx.items ad".len();

        let completion = compute_console_completion_from_descriptors(
            input,
            cursor,
            &descriptors,
            &ConsoleCompletionContext::default(),
        )
        .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["add"]
        );
    }

    #[test]
    fn completes_nested_command_segment_after_domain_root() {
        let descriptors = [
            ConsoleCommandDescriptor {
                name: "scene.stats",
                aliases: &[],
                category: "scene",
                help: "Scene stats.",
                usage: "scene.stats",
                examples: &["scene stats"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scene.entities",
                aliases: &[],
                category: "scene",
                help: "Scene entities.",
                usage: "scene.entities <list|add|remove|inspect> [entity-name]",
                examples: &["scene entities"],
                dev_only: true,
            },
        ];

        let completion = compute_console_completion_from_descriptors(
            "scene ent",
            "scene ent".len(),
            &descriptors,
            &ConsoleCompletionContext::default(),
        )
        .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["entities"]
        );
    }

    #[test]
    fn completes_argument_after_nested_command_name() {
        let descriptors = [ConsoleCommandDescriptor {
            name: "scene.entities",
            aliases: &[],
            category: "scene",
            help: "Scene entities.",
            usage: "scene.entities <list|add|remove|inspect> [entity-name]",
            examples: &["scene.entities list"],
            dev_only: true,
        }];

        let completion = compute_console_completion_from_descriptors(
            "scene.entities ",
            "scene.entities ".len(),
            &descriptors,
            &ConsoleCompletionContext::default(),
        )
        .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["add", "inspect", "list", "remove"]
        );
    }

    #[test]
    fn completes_entity_name_inside_get_entity_string() {
        let context = ConsoleCompletionContext {
            entity_names: vec!["layer1".to_owned(), "layer2".to_owned()],
            ..ConsoleCompletionContext::default()
        };

        let input = "get_entity(\"la\")";
        let cursor = "get_entity(\"la".len();

        let completion = compute_console_completion_from_descriptors(input, cursor, &[], &context)
            .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["layer1", "layer2"]
        );
    }

    #[test]
    fn completes_rhai_builtin_shortcut_at_initial_token() {
        let completion = compute_console_completion_from_descriptors(
            "get_en",
            "get_en".len(),
            &[],
            &ConsoleCompletionContext::default(),
        )
        .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["get_entity"]
        );
        assert_eq!(completion.suggestions[0].insert_text, "get_entity(\"");
    }

    #[test]
    fn completes_declared_rhai_variable_from_context() {
        let context = ConsoleCompletionContext {
            rhai_symbols: vec![ConsoleRhaiSymbol::variable(
                "player",
                ConsoleRhaiValueKind::EntityRef,
            )],
            ..ConsoleCompletionContext::default()
        };

        let completion =
            compute_console_completion_from_descriptors("pla", "pla".len(), &[], &context)
                .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["player"]
        );
    }

    #[test]
    fn completes_rhai_variable_after_assignment_token() {
        let context = ConsoleCompletionContext {
            rhai_symbols: vec![ConsoleRhaiSymbol::variable(
                "player",
                ConsoleRhaiValueKind::EntityRef,
            )],
            ..ConsoleCompletionContext::default()
        };
        let input = "let selected = pla";

        let completion =
            compute_console_completion_from_descriptors(input, input.len(), &[], &context)
                .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["player"]
        );
    }

    #[test]
    fn completes_properties_for_declared_entity_variable() {
        let context = ConsoleCompletionContext {
            rhai_symbols: vec![ConsoleRhaiSymbol::variable(
                "player",
                ConsoleRhaiValueKind::EntityRef,
            )],
            ..ConsoleCompletionContext::default()
        };

        let completion = compute_console_completion_from_descriptors(
            "player.v",
            "player.v".len(),
            &[],
            &context,
        )
        .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["visible"]
        );
    }

    #[test]
    fn completes_properties_for_variable_declared_in_current_line() {
        let input = "let player = get_entity(\"hero\"); player.c";
        let completion = compute_console_completion_from_descriptors(
            input,
            input.len(),
            &[],
            &ConsoleCompletionContext::default(),
        )
        .expect("completion should exist");

        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["collision_enabled"]
        );
    }

    #[test]
    fn extracts_declared_rhai_symbols_from_console_source() {
        let symbols = collect_console_rhai_symbols_from_source(
            "let player = get_entity(\"hero\"); const fx = postfx.item(0)",
        );

        assert_eq!(
            symbols
                .iter()
                .map(|symbol| (symbol.name.as_str(), symbol.value_kind))
                .collect::<Vec<_>>(),
            vec![
                ("player", ConsoleRhaiValueKind::EntityRef),
                ("fx", ConsoleRhaiValueKind::PostFxItem),
            ]
        );
    }
}
