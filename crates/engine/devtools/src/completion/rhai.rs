use std::collections::BTreeMap;

use super::{
    ConsoleCompletionContext, ConsoleCompletionKind, ConsoleCompletionSnapshot,
    ConsoleCompletionSuggestion, ConsoleRhaiSymbol, ConsoleRhaiValueKind, complete_values,
    active_token, sort_and_limit_suggestions,
};

const MAX_COMPLETION_SUGGESTIONS: usize = super::MAX_COMPLETION_SUGGESTIONS;

pub(super) fn complete_rhai_symbol(
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

pub(super) fn rhai_symbol_suggestions(
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

pub(super) fn complete_quoted_entity_name(
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

pub(super) fn complete_rhai_property(
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
