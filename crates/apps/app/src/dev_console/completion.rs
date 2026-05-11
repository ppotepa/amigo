use std::sync::Mutex;

use super::model::ConsoleCommandDescriptor;
use super::registry::ConsoleCommandRegistry;

const MAX_COMPLETION_SUGGESTIONS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConsoleCompletionKind {
    Command,
    Alias,
    Argument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsoleCompletionSuggestion {
    pub(crate) label: String,
    pub(crate) insert_text: String,
    pub(crate) detail: String,
    pub(crate) kind: ConsoleCompletionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsoleCompletionSnapshot {
    pub(crate) input: String,
    pub(crate) replacement_start: usize,
    pub(crate) replacement_end: usize,
    pub(crate) suggestions: Vec<ConsoleCompletionSuggestion>,
    pub(crate) selected_index: usize,
}

impl ConsoleCompletionSnapshot {
    pub(crate) fn is_active(&self) -> bool {
        !self.suggestions.is_empty()
    }

    pub(crate) fn selected(&self) -> Option<&ConsoleCompletionSuggestion> {
        self.suggestions.get(self.selected_index)
    }
}

#[derive(Debug, Default)]
struct ConsoleCompletionInner {
    snapshot: Option<ConsoleCompletionSnapshot>,
}

#[derive(Debug, Default)]
pub(crate) struct ConsoleCompletionState {
    inner: Mutex<ConsoleCompletionInner>,
}

impl ConsoleCompletionState {
    pub(crate) fn snapshot(&self) -> Option<ConsoleCompletionSnapshot> {
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot
            .clone()
    }

    pub(crate) fn clear(&self) {
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot = None;
    }

    pub(crate) fn refresh(&self, input: &str, registry: &ConsoleCommandRegistry) {
        let snapshot = compute_console_completion(input, registry);
        self.inner
            .lock()
            .expect("console completion mutex should not be poisoned")
            .snapshot = snapshot.filter(ConsoleCompletionSnapshot::is_active);
    }

    pub(crate) fn select_next(&self) -> bool {
        self.select_delta(1)
    }

    pub(crate) fn select_previous(&self) -> bool {
        self.select_delta(-1)
    }

    pub(crate) fn accept_tab(&self, input: &str) -> Option<String> {
        let snapshot = self.snapshot()?;
        if snapshot.suggestions.is_empty() {
            return None;
        }

        let replacement = current_replacement(input, &snapshot);
        if snapshot.suggestions.len() == 1 {
            return Some(apply_suggestion(input, &snapshot, &snapshot.suggestions[0]));
        }

        if let Some(prefix) = common_insert_prefix(&snapshot.suggestions) {
            if prefix.len() > replacement.len() && prefix.starts_with(replacement) {
                return Some(apply_insert_text(input, &snapshot, &prefix, false));
            }
        }

        snapshot
            .selected()
            .map(|suggestion| apply_suggestion(input, &snapshot, suggestion))
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

pub(crate) fn compute_console_completion(
    input: &str,
    registry: &ConsoleCommandRegistry,
) -> Option<ConsoleCompletionSnapshot> {
    let descriptors = registry.descriptors();
    let token = active_token(input);
    if token.command_token {
        return complete_command_name(input, token.start, token.end, token.value, &descriptors);
    }
    complete_argument(input, token.start, token.end, token.value, &descriptors)
}

#[derive(Debug, Clone, Copy)]
struct ActiveToken<'a> {
    start: usize,
    end: usize,
    value: &'a str,
    command_token: bool,
}

fn active_token(input: &str) -> ActiveToken<'_> {
    let end = input.len();
    let start = input
        .char_indices()
        .rev()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index + ch.len_utf8()))
        .unwrap_or(0);
    let command_token = !input[..start].contains(char::is_whitespace);
    ActiveToken {
        start,
        end,
        value: &input[start..end],
        command_token,
    }
}

fn complete_command_name(
    input: &str,
    start: usize,
    end: usize,
    prefix: &str,
    descriptors: &[ConsoleCommandDescriptor],
) -> Option<ConsoleCompletionSnapshot> {
    if prefix.trim().is_empty() {
        return None;
    }

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

    suggestions.sort_by(|a, b| a.label.cmp(&b.label));
    suggestions.dedup_by(|a, b| a.label == b.label);
    suggestions.truncate(MAX_COMPLETION_SUGGESTIONS);

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        replacement_start: start,
        replacement_end: end,
        suggestions,
        selected_index: 0,
    })
}

fn complete_argument(
    input: &str,
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

    Some(ConsoleCompletionSnapshot {
        input: input.to_owned(),
        replacement_start: start,
        replacement_end: end,
        suggestions,
        selected_index: 0,
    })
}

fn usage_enum_values(usage: &str, arg_index: usize) -> Option<Vec<&str>> {
    let token = usage.split_whitespace().skip(1 + arg_index).next()?;
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
) -> String {
    apply_insert_text(input, snapshot, &suggestion.insert_text, true)
}

fn apply_insert_text(
    input: &str,
    snapshot: &ConsoleCompletionSnapshot,
    insert_text: &str,
    preserve_spacing: bool,
) -> String {
    let mut next = String::new();
    next.push_str(&input[..snapshot.replacement_start]);
    next.push_str(insert_text);
    if !preserve_spacing && !insert_text.ends_with(' ') {
        next.push(' ');
    }
    next.push_str(&input[snapshot.replacement_end..]);
    next
}

fn current_replacement<'a>(input: &'a str, snapshot: &ConsoleCompletionSnapshot) -> &'a str {
    &input[snapshot.replacement_start..snapshot.replacement_end]
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
    use super::*;
    use crate::dev_console::dispatcher::ConsoleCommandContext;
    use crate::dev_console::model::{
        ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
    };
    use crate::dev_console::registry::{ConsoleCommandHandler, ConsoleCommandRegistry};

    struct TestCommandHandler;

    impl ConsoleCommandHandler for TestCommandHandler {
        fn name(&self) -> &'static str {
            "test"
        }

        fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
            vec![
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
            ]
        }

        fn can_handle(&self, _command: &ParsedConsoleCommand) -> bool {
            false
        }

        fn handle(
            &self,
            _ctx: &ConsoleCommandContext<'_>,
            _command: ParsedConsoleCommand,
        ) -> ConsoleCommandResult {
            ConsoleCommandResult::Silent
        }
    }

    #[test]
    fn completes_command_prefix() {
        let registry = ConsoleCommandRegistry::default();
        registry.register(TestCommandHandler);
        let completion = compute_console_completion("debug.fp", &registry).unwrap();
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
        let registry = ConsoleCommandRegistry::default();
        registry.register(TestCommandHandler);
        let completion = compute_console_completion("debug.fps o", &registry).unwrap();
        assert_eq!(
            completion
                .suggestions
                .iter()
                .map(|suggestion| suggestion.label.as_str())
                .collect::<Vec<_>>(),
            vec!["off", "on"]
        );
    }
}
