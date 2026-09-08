mod validation;

use std::path::PathBuf;

use amigo_codemap_api::{validate_codemap_graph, CodeMapNodeId};
use amigo_plugin_index::{build_codemap_graph_from_index, validate_plugin_index, PluginIndex};
use amigo_plugin_loader::load_plugin_manifests_from_plugins_dir;
use validation::{parse_validate_roots, validate_plugin_tree};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "validate") {
        let roots = parse_validate_roots(&args[1..]);
        validate_or_exit(&roots);
        println!("plugin tree validation passed");
        return;
    }

    let first_tail = args.get(1).cloned();
    let mut args = args.into_iter();
    let first = args.next().unwrap_or_else(|| "summary".to_owned());
    let known_command = matches!(
        first.as_str(),
        "summary" | "check" | "plugins" | "targets" | "diagnostics" | "graph"
    ) && !(first == "plugins" && first_tail.is_some());
    let command = if known_command {
        first.clone()
    } else {
        "check".to_owned()
    };
    let roots: Vec<PathBuf> = if known_command {
        let roots = args.map(PathBuf::from).collect::<Vec<_>>();
        if roots.is_empty() {
            vec![PathBuf::from("plugins")]
        } else {
            roots
        }
    } else {
        std::iter::once(PathBuf::from(first))
            .chain(args.map(PathBuf::from))
            .collect()
    };

    validate_or_exit(&roots);

    let mut manifests = Vec::new();
    for root in &roots {
        match load_plugin_manifests_from_plugins_dir(root) {
            Ok(root_manifests) => manifests.extend(root_manifests),
            Err(errors) => {
                eprintln!("plugin load failed: {errors:#?}");
                std::process::exit(1);
            }
        }
    }

    let index = PluginIndex::from_manifests(manifests);
    if let Err(errors) = validate_plugin_index(&index) {
        eprintln!("plugin index validation failed: {errors:#?}");
        std::process::exit(1);
    }
    let graph = build_codemap_graph_from_index(&index);
    if let Err(errors) = validate_codemap_graph(&graph) {
        eprintln!("codemap graph validation failed: {errors:#?}");
        std::process::exit(1);
    }

    match command.as_str() {
        "summary" | "check" => print_summary(&index, graph.nodes.len(), graph.edges.len()),
        "plugins" => print_plugins(&index),
        "targets" => print_targets(&graph),
        "diagnostics" => print_diagnostics(&graph),
        "graph" => print_graph(&graph),
        other => {
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
}

fn validate_or_exit(roots: &[PathBuf]) {
    if let Err(errors) = validate_plugin_tree(roots) {
        eprintln!("plugin tree validation failed:");
        for error in errors {
            eprintln!("- {error}");
        }
        std::process::exit(1);
    }
}

fn print_summary(index: &PluginIndex, nodes: usize, edges: usize) {
    println!("plugins: {}", index.len());
    println!("nodes: {nodes}");
    println!("edges: {edges}");
}
fn print_plugins(index: &PluginIndex) {
    let mut ids = index
        .manifests()
        .map(|m| m.id.0.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    for id in ids {
        println!("{id}");
    }
}
fn print_targets(graph: &amigo_codemap_api::CodeMapGraph) {
    print_nodes_by_kind(graph, |id| matches!(id, CodeMapNodeId::Target(_)));
}
fn print_diagnostics(graph: &amigo_codemap_api::CodeMapGraph) {
    print_nodes_by_kind(graph, |id| {
        matches!(id, CodeMapNodeId::DiagnosticChannel(_))
    });
}
fn print_graph(graph: &amigo_codemap_api::CodeMapGraph) {
    for edge in &graph.edges {
        println!("{:?} --{:?}--> {:?}", edge.from, edge.kind, edge.to);
    }
}
fn print_nodes_by_kind(
    graph: &amigo_codemap_api::CodeMapGraph,
    predicate: impl Fn(&CodeMapNodeId) -> bool,
) {
    let mut labels = graph
        .nodes
        .values()
        .filter(|node| predicate(&node.id))
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();
    labels.sort_unstable();
    labels.dedup();
    for label in labels {
        println!("{label}");
    }
}
