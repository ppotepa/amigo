use std::path::PathBuf;

use amigo_codemap_api::{validate_codemap_graph, CodeMapNodeId};
use amigo_plugin_index::{
    build_codemap_graph_from_index, validate_plugin_index, PluginIndex,
};
use amigo_plugin_loader::load_plugin_manifests_from_plugins_dir;

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "summary".to_owned());
    let plugins_dir = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("plugins"));

    let manifests = match load_plugin_manifests_from_plugins_dir(&plugins_dir) {
        Ok(manifests) => manifests,
        Err(errors) => {
            eprintln!("plugin load failed: {errors:#?}");
            std::process::exit(1);
        }
    };

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
            eprintln!("usage: amigo-plugin-check [summary|plugins|targets|diagnostics|graph] [plugins_dir]");
            std::process::exit(2);
        }
    }
}

fn print_summary(index: &PluginIndex, nodes: usize, edges: usize) {
    println!("plugins: {}", index.len());
    println!("nodes: {nodes}");
    println!("edges: {edges}");
}

fn print_plugins(index: &PluginIndex) {
    let mut plugin_ids = index
        .manifests()
        .map(|manifest| manifest.id.0.as_str())
        .collect::<Vec<_>>();
    plugin_ids.sort_unstable();

    for plugin_id in plugin_ids {
        println!("{plugin_id}");
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
