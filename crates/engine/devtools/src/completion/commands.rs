use crate::{ConsoleArgKind, ConsoleArgSpec, ConsoleCommandDescriptor, ConsoleCommandSchema};

use super::resources::inspect_target_suggestions;
use super::rhai::rhai_symbol_suggestions;
use super::{
    ConsoleCompletionContext, ConsoleCompletionKind, ConsoleCompletionSnapshot,
    ConsoleCompletionSuggestion, MAX_COMPLETION_SUGGESTIONS, sort_and_limit_suggestions,
};

pub(super) fn complete_initial_token(
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

pub(super) fn complete_command_segment(
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

pub(super) fn complete_typed_argument(
    input: &str,
    cursor_index: usize,
    start: usize,
    end: usize,
    prefix: &str,
    schemas: &[ConsoleCommandSchema],
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    let command_name = input.split_whitespace().next()?;
    let arg_index = input[..start].split_whitespace().skip(1).count();
    let previous_args = input[..start]
        .split_whitespace()
        .skip(1)
        .collect::<Vec<_>>();

    let mut suggestions = Vec::new();
    for schema in schemas
        .iter()
        .filter(|schema| schema.matches_name(command_name))
    {
        for form in schema.forms {
            if !form_matches_previous_args(form, &previous_args) {
                continue;
            }
            let Some(arg_spec) = form.args.get(arg_index) else {
                continue;
            };
            suggestions.extend(suggestions_for_arg_spec(arg_spec, prefix, context));
        }
    }

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

fn form_matches_previous_args(form: &crate::ConsoleCommandForm, previous_args: &[&str]) -> bool {
    for (index, value) in previous_args.iter().enumerate() {
        let Some(arg_spec) = form.args.get(index) else {
            return false;
        };
        if !arg_value_matches_spec(value, arg_spec) {
            return false;
        }
    }
    true
}

fn arg_value_matches_spec(value: &str, spec: &ConsoleArgSpec) -> bool {
    match spec.kind {
        ConsoleArgKind::Literal(values) => values.contains(&value),
        _ => true,
    }
}

fn suggestions_for_arg_spec(
    spec: &ConsoleArgSpec,
    prefix: &str,
    context: &ConsoleCompletionContext,
) -> Vec<ConsoleCompletionSuggestion> {
    match spec.kind {
        ConsoleArgKind::Literal(values) => values
            .iter()
            .copied()
            .filter(|value| value.starts_with(prefix))
            .map(|value| ConsoleCompletionSuggestion {
                label: value.to_owned(),
                insert_text: format!("{value} "),
                detail: spec.name.to_owned(),
                kind: ConsoleCompletionKind::Argument,
            })
            .collect(),
        ConsoleArgKind::EntityName => context
            .entity_names
            .iter()
            .filter(|value| value.starts_with(prefix))
            .map(|value| ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "scene entity".to_owned(),
                kind: ConsoleCompletionKind::Value,
            })
            .collect(),
        ConsoleArgKind::PostFxKind => context
            .postfx_kinds
            .iter()
            .filter(|value| value.starts_with(prefix))
            .map(|value| ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "post-fx kind".to_owned(),
                kind: ConsoleCompletionKind::Value,
            })
            .collect(),
        ConsoleArgKind::PostFxIndex => context
            .postfx_indices
            .iter()
            .filter(|value| value.starts_with(prefix))
            .map(|value| ConsoleCompletionSuggestion {
                label: value.clone(),
                insert_text: format!("{value} "),
                detail: "post-fx item index".to_owned(),
                kind: ConsoleCompletionKind::Value,
            })
            .collect(),
        ConsoleArgKind::InspectTarget => inspect_target_suggestions(prefix, context),
        ConsoleArgKind::Bool => ["true", "false"]
            .into_iter()
            .filter(|value| value.starts_with(prefix))
            .map(|value| ConsoleCompletionSuggestion {
                label: value.to_owned(),
                insert_text: format!("{value} "),
                detail: "bool".to_owned(),
                kind: ConsoleCompletionKind::Value,
            })
            .collect(),
        ConsoleArgKind::Int | ConsoleArgKind::Float | ConsoleArgKind::String => Vec::new(),
    }
}

pub(super) fn complete_argument(
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

fn usage_enum_values(usage: &str, arg_index: usize) -> Option<Vec<&str>> {
    let token = usage.split_whitespace().nth(1 + arg_index)?;
    let token = token
        .trim_matches('[')
        .trim_matches(']')
        .trim_matches('<')
        .trim_matches('>');
    token.contains('|').then(|| token.split('|').collect())
}
