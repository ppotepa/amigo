use amigo_devtools::{
    ConsoleArgKind, ConsoleArgSpec, ConsoleCommandDescriptor, ConsoleCommandForm,
    ConsoleCommandResult, ConsoleCommandSchema, DevConsoleCommandContext, ParsedConsoleCommand,
    RuntimeConsoleCommandHandler,
};
use amigo_scripting_api::ScriptRuntimeService;

use crate::inspect::{
    format_inspect_error, process_pending_inspect_requests, resolve_text_inspect_selector,
};
use crate::runtime_apply::apply_property_value;
use crate::selection::{select_node_by_id, select_viewport_target};
use crate::state::{EditorPropertyValue, IngameEditorState, SelectionSource};
use amigo_editor_authoring::{
    AuthoringNode, AuthoringPropertyEditor, AuthoringPropertyValue, AuthoringRuntimeBinding,
    AuthoringSceneGraph, AuthoringSceneGraphService, build_property_panel_for_node_with_registry,
};

pub struct IngameEditorConsoleCommandHandler;

const INSPECT_ARGS: &[ConsoleArgSpec] = &[ConsoleArgSpec::required(
    "target",
    ConsoleArgKind::InspectTarget,
)];
const INSPECT_FORMS: &[ConsoleCommandForm] = &[ConsoleCommandForm {
    usage: "inspect <target-expression>",
    args: INSPECT_ARGS,
}];

impl RuntimeConsoleCommandHandler for IngameEditorConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "ingame-editor-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "inspect",
                aliases: &["i"],
                category: "editor",
                help: "Open the right-side inspector dock for a real inspectable runtime handle.",
                usage: "inspect <entity(\"name\")|postfx.item(index)|render2d.get_layer(\"id\")|variable|text-selector>",
                examples: &[
                    "inspect entity(\"player\")",
                    "let fx = postfx.item(0)",
                    "inspect fx",
                    "inspect postfx.item(0)",
                    "inspect render2d.get_layer(\"background.city\")",
                    "inspect selected",
                    "inspect entity:player",
                    "inspect postfx:0",
                ],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.toggle",
                aliases: &["editor"],
                category: "editor",
                help: "Toggle ingame editor mockup.",
                usage: "editor.toggle",
                examples: &["editor.toggle", "editor"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.tree",
                aliases: &[],
                category: "editor",
                help: "Show current editor YAML tree summary.",
                usage: "editor.tree",
                examples: &["editor.tree"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.tree.filter",
                aliases: &[],
                category: "editor",
                help: "Filter editor YAML tree by text.",
                usage: "editor.tree.filter <text>",
                examples: &[
                    "editor.tree.filter rain",
                    "editor.tree.filter LayeredImage2D",
                ],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.tree.filter.clear",
                aliases: &[],
                category: "editor",
                help: "Clear editor YAML tree filter.",
                usage: "editor.tree.filter.clear",
                examples: &["editor.tree.filter.clear"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.node",
                aliases: &[],
                category: "editor",
                help: "Show YAML authoring node details.",
                usage: "editor.node <node-id>",
                examples: &["editor.node <node-id>"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.selected",
                aliases: &[],
                category: "editor",
                help: "Show currently selected editor authoring node.",
                usage: "editor.selected",
                examples: &["editor.selected"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.inspect",
                aliases: &[],
                category: "editor",
                help: "Inspect metadata-driven property panel for an authoring node.",
                usage: "editor.inspect [node-id]",
                examples: &["editor.inspect", "editor.inspect <node-id>"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.properties",
                aliases: &[],
                category: "editor",
                help: "List metadata-driven property ids for an authoring node.",
                usage: "editor.properties [node-id]",
                examples: &["editor.properties", "editor.properties <node-id>"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.viewport",
                aliases: &[],
                category: "editor",
                help: "Show editor viewport pan/zoom and selection state.",
                usage: "editor.viewport",
                examples: &["editor.viewport"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.viewport.reset",
                aliases: &[],
                category: "editor",
                help: "Reset editor viewport pan/zoom.",
                usage: "editor.viewport.reset",
                examples: &["editor.viewport.reset"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.selection",
                aliases: &[],
                category: "editor",
                help: "Show selected node and viewport hit selection.",
                usage: "editor.selection",
                examples: &["editor.selection"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.preview.opacity",
                aliases: &[],
                category: "editor",
                help: "Show current runtime opacity for key rotten-club preview layers.",
                usage: "editor.preview.opacity",
                examples: &["editor.preview.opacity"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.preview.reveal",
                aliases: &[],
                category: "editor",
                help: "Debug-only: force common rotten-club preview layers visible.",
                usage: "editor.preview.reveal",
                examples: &["editor.preview.reveal"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.hit",
                aliases: &[],
                category: "editor",
                help: "Select a viewport target by logical game viewport coordinates.",
                usage: "editor.hit <x> <y>",
                examples: &["editor.hit 640 360"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.tree.expand_all",
                aliases: &[],
                category: "editor",
                help: "Expand all nodes in the editor YAML tree.",
                usage: "editor.tree.expand_all",
                examples: &["editor.tree.expand_all"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.tree.collapse_all",
                aliases: &[],
                category: "editor",
                help: "Collapse all expandable nodes in the editor YAML tree.",
                usage: "editor.tree.collapse_all",
                examples: &["editor.tree.collapse_all"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.source",
                aliases: &[],
                category: "editor",
                help: "List YAML source files in current authoring graph.",
                usage: "editor.source",
                examples: &["editor.source"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.reload",
                aliases: &[],
                category: "editor",
                help: "Invalidate editor authoring graph cache.",
                usage: "editor.reload",
                examples: &["editor.reload"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.select",
                aliases: &[],
                category: "editor",
                help: "Select editor YAML node by id.",
                usage: "editor.select <node-id>",
                examples: &[
                    "editor.select mods/rotten-club/scenes/main-menu/visual/render.yml#/visual2d/render_layers/0",
                ],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.layer.opacity",
                aliases: &[],
                category: "editor",
                help: "Set render layer opacity in runtime preview.",
                usage: "editor.layer.opacity <layer-id> <value>",
                examples: &["editor.layer.opacity background.city 0.5"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "editor.layered.opacity",
                aliases: &[],
                category: "editor",
                help: "Set layered image layer opacity in runtime preview.",
                usage: "editor.layered.opacity <entity> <layer-id> <value>",
                examples: &["editor.layered.opacity background club_sign 1.0"],
                dev_only: true,
            },
        ]
    }

    fn schemas(&self) -> Vec<ConsoleCommandSchema> {
        vec![ConsoleCommandSchema {
            command_name: "inspect",
            aliases: &["i"],
            forms: INSPECT_FORMS,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "inspect"
            || command.name == "i"
            || command.name == "editor"
            || command.name.starts_with("editor.")
    }

    fn handle(
        &self,
        ctx: &DevConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let state = match ctx.required::<IngameEditorState>() {
            Ok(state) => state,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match command.name.as_str() {
            "inspect" | "i" => handle_inspect_command(&ctx, state.as_ref(), &command),
            "editor" | "editor.toggle" => {
                state.toggle();
                ConsoleCommandResult::ok(if state.is_open() {
                    "editor opened"
                } else {
                    "editor closed"
                })
            }
            "editor.open" => {
                state.set_open(true);
                ConsoleCommandResult::ok("editor opened")
            }
            "editor.close" => {
                state.set_open(false);
                ConsoleCommandResult::ok("editor closed")
            }
            "editor.select" => {
                let Some(node_id) = command.args.first() else {
                    return ConsoleCommandResult::error("usage: editor.select <node-id>");
                };
                match current_graph(ctx) {
                    Ok(graph) => {
                        if select_node_by_id(
                            ctx.runtime,
                            state.as_ref(),
                            &graph,
                            node_id.clone(),
                            None,
                            None,
                            SelectionSource::Command,
                        ) {
                            ConsoleCommandResult::ok(format!("selected {node_id}"))
                        } else {
                            ConsoleCommandResult::error(format!("unknown node `{node_id}`"))
                        }
                    }
                    Err(error) => ConsoleCommandResult::error(error),
                }
            }
            "editor.tree" => match current_graph(ctx) {
                Ok(graph) => ConsoleCommandResult::ok(format!(
                    "authoring tree: mod={} scene={} files={} root={}",
                    graph.source_mod,
                    graph.scene_id,
                    graph.source_files.len(),
                    graph.root_file.display()
                )),
                Err(error) => ConsoleCommandResult::error(error),
            },
            "editor.tree.filter" => {
                if command.args.is_empty() {
                    return ConsoleCommandResult::error("usage: editor.tree.filter <text>");
                }
                let filter = command.args.join(" ");
                state.set_tree_filter(filter.clone());
                ConsoleCommandResult::ok(format!("editor tree filter set: {filter}"))
            }
            "editor.tree.filter.clear" => {
                state.clear_tree_filter();
                ConsoleCommandResult::ok("editor tree filter cleared")
            }
            "editor.source" => match current_graph(ctx) {
                Ok(graph) => {
                    let files = graph
                        .source_files
                        .iter()
                        .map(|path| format!("- {}", path.display()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    ConsoleCommandResult::ok(files)
                }
                Err(error) => ConsoleCommandResult::error(error),
            },
            "editor.node" => {
                let Some(node_id) = command.args.first() else {
                    return ConsoleCommandResult::error("usage: editor.node <node-id>");
                };

                match current_graph(ctx) {
                    Ok(graph) => match graph.find_node(node_id) {
                        Some(node) => ConsoleCommandResult::ok(format_node_details(node)),
                        None => ConsoleCommandResult::error(format!("unknown node `{node_id}`")),
                    },
                    Err(error) => ConsoleCommandResult::error(error),
                }
            }
            "editor.selected" => {
                let graph = match current_graph(ctx) {
                    Ok(graph) => graph,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                let snapshot = state.snapshot();
                let Some(node_id) = snapshot.selection.as_ref().map(|s| s.node_id.as_str()) else {
                    return ConsoleCommandResult::error("no editor node selected");
                };
                match graph.find_node(node_id) {
                    Some(node) => ConsoleCommandResult::ok(format_node_details(node)),
                    None => {
                        ConsoleCommandResult::error(format!("unknown selected node `{node_id}`"))
                    }
                }
            }
            "editor.inspect" => {
                let node_id = match requested_or_selected_node_id(&command, state.as_ref()) {
                    Ok(node_id) => node_id,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                let selector = format!("node:{node_id}");
                match resolve_text_inspect_selector(ctx.runtime, state.as_ref(), &selector) {
                    Ok(resolved) => {
                        let label = resolved.target.label.clone();
                        state.open_inspector_dock(resolved.target, resolved.selection);
                        ctx.console.set_open(false);
                        ConsoleCommandResult::ok(format!("opened inspector: {label}"))
                    }
                    Err(error) => ConsoleCommandResult::error(format_inspect_error(error)),
                }
            }
            "editor.inspect.dump" => {
                let graph = match current_graph(ctx) {
                    Ok(graph) => graph,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                let node_id = match requested_or_selected_node_id(&command, state.as_ref()) {
                    Ok(node_id) => node_id,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                match graph.find_node(&node_id) {
                    Some(node) => ConsoleCommandResult::ok(format_inspection(ctx.runtime, node)),
                    None => ConsoleCommandResult::error(format!("unknown node `{node_id}`")),
                }
            }
            "editor.properties" => {
                let graph = match current_graph(ctx) {
                    Ok(graph) => graph,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                let node_id = match requested_or_selected_node_id(&command, state.as_ref()) {
                    Ok(node_id) => node_id,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                match graph.find_node(&node_id) {
                    Some(node) => ConsoleCommandResult::ok(format_properties(ctx.runtime, node)),
                    None => ConsoleCommandResult::error(format!("unknown node `{node_id}`")),
                }
            }
            "editor.viewport" => {
                let snapshot = state.snapshot();
                ConsoleCommandResult::ok(format!(
                    "viewport pan=({:.1},{:.1}) zoom={:.3} panning={} selection={}",
                    snapshot.viewport_pan_x,
                    snapshot.viewport_pan_y,
                    snapshot.viewport_zoom,
                    snapshot.is_panning_viewport,
                    format_viewport_selection(snapshot.selection.as_ref())
                ))
            }
            "editor.viewport.reset" => {
                state.reset_viewport_view();
                ConsoleCommandResult::ok("editor viewport reset")
            }
            "editor.selection" => {
                let snapshot = state.snapshot();
                ConsoleCommandResult::ok(format!(
                    "selected_node={} source={} pointer={} viewport={}",
                    snapshot
                        .selection
                        .as_ref()
                        .map(|s| s.node_id.as_str())
                        .unwrap_or("<none>"),
                    snapshot
                        .selection
                        .as_ref()
                        .and_then(|s| s.source_path.as_deref())
                        .unwrap_or("<none>"),
                    snapshot
                        .selection
                        .as_ref()
                        .and_then(|s| s.yaml_pointer.as_deref())
                        .unwrap_or("<none>"),
                    format_viewport_selection(snapshot.selection.as_ref())
                ))
            }
            "editor.preview.opacity" => preview_opacity_report(ctx),
            "editor.preview.reveal" => preview_reveal(ctx),
            "editor.hit" => {
                let [x, y] = command.args.as_slice() else {
                    return ConsoleCommandResult::error("usage: editor.hit <x> <y>");
                };
                let Ok(x) = x.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid x `{x}`"));
                };
                let Ok(y) = y.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid y `{y}`"));
                };
                let graph = match current_graph(ctx) {
                    Ok(graph) => graph,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                if select_viewport_target(ctx.runtime, state.as_ref(), &graph, x, y) {
                    ConsoleCommandResult::ok(format!("viewport hit selected at {x:.1},{y:.1}"))
                } else {
                    ConsoleCommandResult::error("no viewport target found")
                }
            }
            "editor.tree.expand_all" => {
                state.expand_all();
                ConsoleCommandResult::ok("editor tree expanded")
            }
            "editor.tree.collapse_all" => {
                let graph = match current_graph(ctx) {
                    Ok(graph) => graph,
                    Err(error) => return ConsoleCommandResult::error(error),
                };
                let mut node_ids = Vec::new();
                collect_collapsible_node_ids(&graph.nodes, &mut node_ids);
                state.collapse_all(node_ids);
                ConsoleCommandResult::ok("editor tree collapsed")
            }
            "editor.reload" => {
                let Some(service) = ctx.runtime.resolve::<AuthoringSceneGraphService>() else {
                    return ConsoleCommandResult::error("authoring service missing");
                };
                service.invalidate_all();
                ConsoleCommandResult::ok("authoring graph cache invalidated")
            }
            "editor.layer.opacity" => {
                let [layer_id, value] = command.args.as_slice() else {
                    return ConsoleCommandResult::error(
                        "usage: editor.layer.opacity <layer-id> <value>",
                    );
                };
                let Ok(value) = value.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid opacity `{value}`"));
                };

                match apply_property_value(
                    ctx.runtime,
                    state.as_ref(),
                    &format!("render_layer.{layer_id}.opacity"),
                    Some(&AuthoringRuntimeBinding::RenderLayerOpacity {
                        layer_id: layer_id.clone(),
                    }),
                    EditorPropertyValue::Number(value),
                ) {
                    Ok(result) => ConsoleCommandResult::ok(format!(
                        "layer `{layer_id}` opacity={value} {result:?}"
                    )),
                    Err(error) => ConsoleCommandResult::error(error.to_string()),
                }
            }
            "editor.layered.opacity" => {
                let [entity, layer_id, value] = command.args.as_slice() else {
                    return ConsoleCommandResult::error(
                        "usage: editor.layered.opacity <entity> <layer-id> <value>",
                    );
                };
                let Ok(value) = value.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid opacity `{value}`"));
                };

                match apply_property_value(
                    ctx.runtime,
                    state.as_ref(),
                    &format!("layered.{entity}.{layer_id}.opacity"),
                    Some(&AuthoringRuntimeBinding::LayeredImageLayerOpacity {
                        entity_name: entity.clone(),
                        layer_id: layer_id.clone(),
                    }),
                    EditorPropertyValue::Number(value),
                ) {
                    Ok(result) => ConsoleCommandResult::ok(format!(
                        "layered `{entity}` layer `{layer_id}` opacity={value} {result:?}"
                    )),
                    Err(error) => ConsoleCommandResult::error(error.to_string()),
                }
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn inspect_tail(command: &ParsedConsoleCommand) -> String {
    command
        .raw
        .split_once(char::is_whitespace)
        .map(|(_, tail)| tail.trim().to_owned())
        .unwrap_or_else(|| command.args.join(" "))
}

fn is_text_inspect_selector(tail: &str) -> bool {
    tail == "selected"
        || tail.starts_with("entity:")
        || tail.starts_with("postfx:")
        || tail.starts_with("layer:")
        || tail.starts_with("render-layer:")
        || tail.starts_with("node:")
}

fn handle_inspect_command(
    ctx: &DevConsoleCommandContext<'_>,
    state: &IngameEditorState,
    command: &ParsedConsoleCommand,
) -> ConsoleCommandResult {
    let tail = inspect_tail(command);
    if tail.is_empty() {
        return ConsoleCommandResult::error(
            "usage: inspect <entity(\"name\")|postfx.item(index)|render2d.get_layer(\"id\")|variable>",
        );
    }

    if is_text_inspect_selector(&tail) {
        return match resolve_text_inspect_selector(ctx.runtime, state, &tail) {
            Ok(resolved) => {
                let label = resolved.target.label.clone();
                state.open_inspector_dock(resolved.target, resolved.selection);
                ctx.console.set_open(false);
                ConsoleCommandResult::ok(format!("opened inspector: {label}"))
            }
            Err(error) => ConsoleCommandResult::error(format_inspect_error(error)),
        };
    }

    let source = format!("inspect({tail})");
    let Some(script_runtime) = ctx.runtime.resolve::<ScriptRuntimeService>() else {
        return ConsoleCommandResult::error("inspect: script runtime unavailable");
    };
    let scene_id = ctx
        .runtime
        .resolve::<amigo_scene::SceneService>()
        .and_then(|scene| scene.selected_scene())
        .map(|scene| scene.as_str().to_owned());
    let context = amigo_scripting_api::DevConsoleScriptContext::new(scene_id);
    if let Err(error) = script_runtime.eval_console(context, &source) {
        return ConsoleCommandResult::error(format!("inspect: expression failed: {error}"));
    }

    match process_pending_inspect_requests(ctx.runtime, state) {
        Ok(Some(target)) => {
            ctx.console.set_open(false);
            ConsoleCommandResult::ok(format!("opened inspector: {}", target.label))
        }
        Ok(None) => {
            ConsoleCommandResult::error("inspect: expression did not produce an inspectable object")
        }
        Err(error) => ConsoleCommandResult::error(format_inspect_error(error)),
    }
}

fn current_graph(ctx: &DevConsoleCommandContext<'_>) -> Result<AuthoringSceneGraph, String> {
    let Some(service) = ctx.runtime.resolve::<AuthoringSceneGraphService>() else {
        return Err("authoring service missing".to_owned());
    };
    service
        .graph_for_current_scene(ctx.runtime)
        .map_err(|error| error.to_string())
}

fn requested_or_selected_node_id(
    command: &ParsedConsoleCommand,
    state: &IngameEditorState,
) -> Result<String, String> {
    if let Some(node_id) = command.args.first() {
        return Ok(node_id.clone());
    }
    state
        .snapshot()
        .selection
        .map(|s| s.node_id)
        .ok_or_else(|| format!("usage: {} [node-id] or select a node first", command.name))
}

fn format_node_details(node: &AuthoringNode) -> String {
    node.summary().to_lines().join("\n")
}

fn format_inspection(runtime: &amigo_runtime::Runtime, node: &AuthoringNode) -> String {
    let registry = crate::component_registry::editor_component_registry(runtime);
    let panel = build_property_panel_for_node_with_registry(node, &registry);
    let property_count: usize = panel
        .groups
        .iter()
        .map(|group| group.properties.len())
        .sum();
    let mut lines = format_node_details(node)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    lines.push(format!("panel: {}", panel.title));
    lines.push(format!("groups: {}", panel.groups.len()));
    lines.push(format!("properties: {property_count}"));
    for group in panel.groups {
        lines.push(format!("group {} ({})", group.id, group.title));
        for property in group.properties {
            lines.push(format!(
                "- {} | {} | value={} | editor={} | binding={}",
                property.id,
                property.label,
                format_property_value(&property.value),
                format_property_editor(&property.editor),
                format_binding(property.binding.as_ref()),
            ));
        }
    }
    lines.join("\n")
}

fn format_properties(runtime: &amigo_runtime::Runtime, node: &AuthoringNode) -> String {
    let registry = crate::component_registry::editor_component_registry(runtime);
    let panel = build_property_panel_for_node_with_registry(node, &registry);
    let mut lines = vec![format!("properties for {}", node.id)];
    for group in panel.groups {
        lines.push(format!("[{}] {}", group.id, group.title));
        for property in group.properties {
            lines.push(format!(
                "{} | {} | editor={} | binding={}",
                property.id,
                property.label,
                format_property_editor(&property.editor),
                format_binding(property.binding.as_ref()),
            ));
        }
    }
    lines.join("\n")
}

fn format_property_value(value: &AuthoringPropertyValue) -> String {
    match value {
        AuthoringPropertyValue::Text(value) => value.clone(),
        AuthoringPropertyValue::Number(value) => format!("{value:.3}"),
        AuthoringPropertyValue::Bool(value) => value.to_string(),
        AuthoringPropertyValue::AssetRef(value) => format!("asset:{value}"),
        AuthoringPropertyValue::Enum(value) => value.clone(),
        AuthoringPropertyValue::Vec2(x, y) => format!("({x:.3}, {y:.3})"),
        AuthoringPropertyValue::Vec3(x, y, z) => format!("({x:.3}, {y:.3}, {z:.3})"),
        AuthoringPropertyValue::Color(value) => value.clone(),
        AuthoringPropertyValue::Empty => "<empty>".to_owned(),
        AuthoringPropertyValue::Unsupported(value) => format!("unsupported:{value}"),
    }
}

fn format_property_editor(editor: &AuthoringPropertyEditor) -> String {
    match editor {
        AuthoringPropertyEditor::ReadOnly => "readonly".to_owned(),
        AuthoringPropertyEditor::Text => "text".to_owned(),
        AuthoringPropertyEditor::Number => "number".to_owned(),
        AuthoringPropertyEditor::Slider { min, max, step } => {
            format!("slider({min:.3}..{max:.3}, step={step:.3})")
        }
        AuthoringPropertyEditor::Toggle => "toggle".to_owned(),
        AuthoringPropertyEditor::AssetPicker { domain } => format!("asset-picker({domain})"),
        AuthoringPropertyEditor::Enum { options } => format!("enum({} options)", options.len()),
        AuthoringPropertyEditor::Color => "color".to_owned(),
        AuthoringPropertyEditor::Vec2 => "vec2".to_owned(),
        AuthoringPropertyEditor::Vec3 => "vec3".to_owned(),
    }
}

fn format_binding(binding: Option<&AuthoringRuntimeBinding>) -> String {
    match binding {
        Some(binding) => format!("{binding:?}"),
        None => "none".to_owned(),
    }
}

fn format_viewport_selection(selection: Option<&crate::state::EditorSelection>) -> String {
    let Some(selection) = selection else {
        return "none".to_owned();
    };
    format!(
        "{} source={:?} logical=({},{})",
        selection.node_id,
        selection.source,
        selection
            .logical_x
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "-".to_owned()),
        selection
            .logical_y
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "-".to_owned())
    )
}

fn collect_collapsible_node_ids(nodes: &[AuthoringNode], out: &mut Vec<String>) {
    for node in nodes {
        if !node.children.is_empty() {
            out.push(node.id.clone());
        }
        collect_collapsible_node_ids(&node.children, out);
    }
}

fn preview_opacity_report(ctx: &DevConsoleCommandContext<'_>) -> ConsoleCommandResult {
    let mut lines = Vec::new();

    if let Ok(layers) = ctx.required::<amigo_2d_composition::RenderLayer2dSceneService>() {
        let commands = layers.commands();
        for id in [
            "background.city",
            "weather.rain.far",
            "weather.rain.mid",
            "weather.rain.near",
            "weather.rain.front",
        ] {
            if let Some(layer) = commands.iter().find(|layer| layer.id == id) {
                lines.push(format!(
                    "render_layer {id}: visible={} opacity={:.3} order={:.1}",
                    layer.visible, layer.opacity, layer.order
                ));
            }
        }
    }

    if let Ok(layered) = ctx.required::<amigo_layered_image_2d_plugin::LayeredImageSceneService>() {
        if let Some(command) = layered
            .commands()
            .into_iter()
            .find(|command| command.entity_name == "background")
        {
            lines.push(format!(
                "layered background: base_opacity={:.3}",
                command.image.base_opacity
            ));
            for layer in [
                "club_sign",
                "club_sign_blur",
                "bar_sign",
                "bar_lanterns",
                "skyline",
            ] {
                let opacity = command
                    .image
                    .layer_overrides
                    .iter()
                    .find(|override_layer| override_layer.id == layer)
                    .and_then(|override_layer| override_layer.opacity)
                    .unwrap_or(1.0);
                lines.push(format!("layered background.{layer}: opacity={opacity:.3}"));
            }
        }
    }

    if lines.is_empty() {
        ConsoleCommandResult::error("no preview opacity services available")
    } else {
        ConsoleCommandResult::ok(lines.join("\n"))
    }
}

fn preview_reveal(ctx: &DevConsoleCommandContext<'_>) -> ConsoleCommandResult {
    let mut changed = Vec::new();

    if let Ok(layers) = ctx.required::<amigo_2d_composition::RenderLayer2dSceneService>() {
        for id in [
            "background.city",
            "weather.rain.far",
            "weather.rain.mid",
            "weather.rain.near",
            "weather.rain.front",
        ] {
            if layers.set_visible(id, true) {
                layers.set_opacity(id, 1.0);
                changed.push(format!("render_layer {id}"));
            }
        }
    }

    if let Ok(layered) = ctx.required::<amigo_layered_image_2d_plugin::LayeredImageSceneService>() {
        if layered.set_base_opacity("background", 1.0) {
            changed.push("layered background base".to_owned());
        }
        for layer in [
            "club_sign",
            "club_sign_blur",
            "bar_sign",
            "bar_sign_blur",
            "pharmacy_cross",
            "pharmacy_cross_blur",
            "bar_lanterns",
            "bar_lanterns_blur",
            "skyline",
            "skyline_blur",
            "club_entry",
            "club_entry_blur",
        ] {
            if layered.set_layer_opacity("background", layer, 1.0) {
                changed.push(format!("background.{layer}"));
            }
        }
    }

    ConsoleCommandResult::ok(format!("preview reveal changed {} values", changed.len()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn inspect_tail_preserves_quotes() {
        let cmd = amigo_devtools::parse_console_command("inspect entity(\"weather.rain.front\")")
            .expect("command");
        assert_eq!(super::inspect_tail(&cmd), "entity(\"weather.rain.front\")");
    }
}
