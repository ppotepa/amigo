use crate::{ConsoleArgKind, ConsoleArgSpec, ConsoleCommandDescriptor, ConsoleCommandSchema};

mod commands;
mod model;
mod provider;
mod resources;
mod rhai;
mod state;

pub use model::{
    ConsoleCompletionContext, ConsoleCompletionEdit, ConsoleCompletionKind,
    ConsoleCompletionSnapshot, ConsoleCompletionSuggestion, ConsoleRhaiSymbol,
    ConsoleRhaiValueKind,
};
pub use provider::{ConsoleCompletionProvider, ConsoleCompletionProviderRegistry};
pub use rhai::collect_console_rhai_symbols_from_source;
pub use state::ConsoleCompletionState;

use rhai::{
    complete_quoted_entity_name, complete_rhai_property, complete_rhai_symbol,
    rhai_symbol_suggestions,
};
use resources::{
    complete_runtime_context, complete_runtime_control_path, inspect_target_suggestions,
};
use commands::{
    complete_argument, complete_command_segment, complete_initial_token, complete_typed_argument,
};

const MAX_COMPLETION_SUGGESTIONS: usize = 8;

pub fn compute_console_completion_from_descriptors(
    input: &str,
    cursor_index: usize,
    descriptors: &[ConsoleCommandDescriptor],
    schemas: &[ConsoleCommandSchema],
    context: &ConsoleCompletionContext,
) -> Option<ConsoleCompletionSnapshot> {
    let cursor_index = clamp_to_char_boundary(input, cursor_index.min(input.len()));

    if let Some(snapshot) = complete_runtime_context(input, cursor_index, context) {
        return Some(snapshot);
    }

    if let Some(snapshot) = complete_runtime_control_path(input, cursor_index, context) {
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

    if let Some(snapshot) = complete_typed_argument(
        input,
        cursor_index,
        token.start,
        token.end,
        token.value,
        schemas,
        context,
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
    use std::sync::Arc;

    use amigo_runtime_control::{
        ControlValue, ControlValueType, RuntimeControlProperty, RuntimeControlProvider,
        RuntimeControlRegistry, RuntimeControlService, RuntimeControlTarget,
    };

    use super::{
        ConsoleArgKind, ConsoleArgSpec, ConsoleCommandDescriptor, ConsoleCommandSchema,
        ConsoleCompletionContext, ConsoleRhaiSymbol, ConsoleRhaiValueKind,
        collect_console_rhai_symbols_from_source, compute_console_completion_from_descriptors,
    };

    struct MockRuntimeControlProvider;

    impl RuntimeControlProvider for MockRuntimeControlProvider {
        fn provider_id(&self) -> &'static str {
            "mock"
        }

        fn rebuild_registry(
            &self,
            registry: &mut RuntimeControlRegistry,
        ) -> Result<(), amigo_runtime_control::RuntimeControlError> {
            registry.register_target(RuntimeControlTarget {
                console_path: "world.weather.rain.front".to_owned(),
                source_id: None,
                label: "rain-front".to_owned(),
                components: vec!["ParticleEmitter2D".to_owned()],
                aliases: Vec::new(),
                source_file: None,
            });
            registry.register_property(RuntimeControlProperty {
                console_path: "world.weather.rain.front.ParticleEmitter2D.spawn_rate".to_owned(),
                target_path: "world.weather.rain.front".to_owned(),
                component: Some("ParticleEmitter2D".to_owned()),
                property_path: "spawn_rate".to_owned(),
                value_type: ControlValueType::F32,
                range: None,
                writable: true,
                readable: true,
                animatable: true,
                source_file: None,
                source_pointer: None,
                provider_id: "mock".to_owned(),
                description: None,
            });
            Ok(())
        }

        fn get(
            &self,
            _path: &RuntimeControlProperty,
        ) -> Result<ControlValue, amigo_runtime_control::RuntimeControlError> {
            Ok(ControlValue::F64(1.0))
        }

        fn set(
            &self,
            _path: &RuntimeControlProperty,
            _value: ControlValue,
        ) -> Result<(), amigo_runtime_control::RuntimeControlError> {
            Ok(())
        }
    }

    fn runtime_control_context() -> ConsoleCompletionContext {
        let service = Arc::new(RuntimeControlService::default());
        service.register_provider(Arc::new(MockRuntimeControlProvider));
        ConsoleCompletionContext {
            runtime_control: Some(service),
            ..ConsoleCompletionContext::default()
        }
    }

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
            &[],
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
            &[],
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
            &[],
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

        let completion =
            compute_console_completion_from_descriptors(input, cursor, &[], &[], &context)
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
            compute_console_completion_from_descriptors("pla", "pla".len(), &[], &[], &context)
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
            compute_console_completion_from_descriptors(input, input.len(), &[], &[], &context)
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

    #[test]
    fn typed_schema_completes_postfx_kind_after_add_action() {
        const ADD_ARGS: &[ConsoleArgSpec] = &[
            ConsoleArgSpec::required("action", ConsoleArgKind::Literal(&["add"])),
            ConsoleArgSpec::required("kind", ConsoleArgKind::PostFxKind),
        ];
        const FORMS: &[crate::ConsoleCommandForm] = &[crate::ConsoleCommandForm {
            usage: "postfx.items add <kind>",
            args: ADD_ARGS,
        }];
        const SCHEMAS: &[ConsoleCommandSchema] = &[ConsoleCommandSchema {
            command_name: "postfx.items",
            aliases: &[],
            forms: FORMS,
        }];

        let context = ConsoleCompletionContext {
            postfx_kinds: vec!["blur".to_owned(), "rain_glass".to_owned()],
            ..ConsoleCompletionContext::default()
        };

        let input = "postfx.items add r";
        let completion =
            compute_console_completion_from_descriptors(input, input.len(), &[], SCHEMAS, &context)
                .expect("completion should exist");

        assert_eq!(completion.suggestions[0].label, "rain_glass");
    }

    #[test]
    fn inspect_target_completion_prefers_expression_first_values() {
        const INSPECT_ARGS: &[ConsoleArgSpec] = &[ConsoleArgSpec::required(
            "target",
            ConsoleArgKind::InspectTarget,
        )];
        const FORMS: &[crate::ConsoleCommandForm] = &[crate::ConsoleCommandForm {
            usage: "inspect <target-expression>",
            args: INSPECT_ARGS,
        }];
        const SCHEMAS: &[ConsoleCommandSchema] = &[ConsoleCommandSchema {
            command_name: "inspect",
            aliases: &["i"],
            forms: FORMS,
        }];

        let context = ConsoleCompletionContext {
            entity_names: vec!["player".to_owned()],
            postfx_indices: vec!["0".to_owned()],
            render_layer_ids: vec!["background.city".to_owned()],
            ..ConsoleCompletionContext::default()
        };

        let completion =
            compute_console_completion_from_descriptors("inspect ", 8, &[], SCHEMAS, &context)
                .expect("completion should exist");

        let labels = completion
            .suggestions
            .iter()
            .map(|suggestion| suggestion.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"selected"));
        assert!(labels.contains(&"entity(\"player\")"));
        assert!(labels.contains(&"postfx.item(0)"));
        assert!(labels.contains(&"render2d.get_layer(\"background.city\")"));
    }

    #[test]
    fn completes_world_root_from_runtime_control() {
        let completion = compute_console_completion_from_descriptors(
            "world.",
            "world.".len(),
            &[],
            &[],
            &runtime_control_context(),
        )
        .expect("completion should exist");

        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "weather")
        );
    }

    #[test]
    fn completes_component_properties_from_runtime_control() {
        let completion = compute_console_completion_from_descriptors(
            "world.weather.rain.front.ParticleEmitter2D.",
            "world.weather.rain.front.ParticleEmitter2D.".len(),
            &[],
            &[],
            &runtime_control_context(),
        )
        .expect("completion should exist");

        assert!(
            completion
                .suggestions
                .iter()
                .any(|suggestion| suggestion.label == "spawn_rate")
        );
    }
}
