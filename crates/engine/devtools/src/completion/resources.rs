use amigo_runtime_control::{RuntimeControlCompletionKind, split_console_prefix};

use super::{
    ConsoleCompletionContext, ConsoleCompletionKind, ConsoleCompletionSnapshot,
    ConsoleCompletionSuggestion, MAX_COMPLETION_SUGGESTIONS, active_token, complete_values,
};
use super::rhai::complete_quoted_entity_name;

pub(super) fn inspect_target_suggestions(
    prefix: &str,
    context: &ConsoleCompletionContext,
) -> Vec<ConsoleCompletionSuggestion> {
    let mut suggestions = Vec::new();
    let selected = "selected".to_owned();
    if selected.starts_with(prefix) {
        suggestions.push(ConsoleCompletionSuggestion {
            label: selected.clone(),
            insert_text: format!("{selected} "),
            detail: "current editor selection".to_owned(),
            kind: ConsoleCompletionKind::Resource,
        });
    }
    for name in &context.entity_names {
        let value = format!("entity(\"{}\")", name.replace('"', "\\\""));
        if value.starts_with(prefix) {
            suggestions.push(ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "inspect entity handle".to_owned(),
                kind: ConsoleCompletionKind::Resource,
            });
        }
    }
    for index in &context.postfx_indices {
        let value = format!("postfx.item({index})");
        if value.starts_with(prefix) {
            suggestions.push(ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "inspect frame post-fx item".to_owned(),
                kind: ConsoleCompletionKind::Resource,
            });
        }
    }
    for id in &context.render_layer_ids {
        let value = format!("render2d.get_layer(\"{}\")", id.replace('"', "\\\""));
        if value.starts_with(prefix) {
            suggestions.push(ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "inspect render layer handle".to_owned(),
                kind: ConsoleCompletionKind::Resource,
            });
        }
    }
    for name in &context.entity_names {
        let value = format!("entity:{name}");
        if value.starts_with(prefix) {
            suggestions.push(ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "inspect target".to_owned(),
                kind: ConsoleCompletionKind::Resource,
            });
        }
    }
    for index in &context.postfx_indices {
        let value = format!("postfx:{index}");
        if value.starts_with(prefix) {
            suggestions.push(ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "inspect target".to_owned(),
                kind: ConsoleCompletionKind::Resource,
            });
        }
    }
    suggestions
}

pub(super) fn complete_runtime_context(
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

pub(super) fn complete_runtime_control_path(
    input: &str,
    cursor_index: usize,
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    let runtime_control = context.runtime_control.as_ref()?;
    let prefix = &input[..cursor_index.min(input.len())];
    let trimmed = prefix.trim_start();
    if !(trimmed == "world" || trimmed.starts_with("world.")) {
        return None;
    }

    let suggestions = runtime_control
        .complete(input, cursor_index)
        .into_iter()
        .map(|entry| ConsoleCompletionSuggestion {
            label: entry.label,
            insert_text: entry.insert_text,
            detail: entry.detail.unwrap_or_else(|| "runtime control".to_owned()),
            kind: runtime_control_kind(entry.kind),
        })
        .take(MAX_COMPLETION_SUGGESTIONS)
        .collect::<Vec<_>>();
    if suggestions.is_empty() {
        return None;
    }

    let replacement_start = if prefix.trim_end().ends_with('.') {
        cursor_index
    } else {
        split_console_prefix(prefix)
            .map(|(_, tail)| cursor_index.saturating_sub(tail.len()))
            .unwrap_or(0)
    };

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        cursor_index,
        replacement_start,
        replacement_end: cursor_index,
        suggestions,
        selected_index: 0,
    })
}

fn runtime_control_kind(kind: RuntimeControlCompletionKind) -> ConsoleCompletionKind {
    match kind {
        RuntimeControlCompletionKind::Namespace => ConsoleCompletionKind::Namespace,
        RuntimeControlCompletionKind::Target => ConsoleCompletionKind::Value,
        RuntimeControlCompletionKind::Component => ConsoleCompletionKind::Resource,
        RuntimeControlCompletionKind::Property => ConsoleCompletionKind::Property,
        RuntimeControlCompletionKind::Method => ConsoleCompletionKind::Function,
        RuntimeControlCompletionKind::Asset => ConsoleCompletionKind::Resource,
    }
}
