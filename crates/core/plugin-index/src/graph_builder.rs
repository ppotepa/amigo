use amigo_codemap_api::{
    CodeMapEdge, CodeMapEdgeKind, CodeMapGraph, CodeMapNode, CodeMapNodeId,
    CodeMapNodeKind,
};
use amigo_plugin_api::{
    CapabilityId, DiagnosticChannelId, DomainId, PluginManifest, SlotId,
    TargetId,
};

use crate::index::PluginIndex;

pub fn build_codemap_graph_from_index(index: &PluginIndex) -> CodeMapGraph {
    let mut graph = CodeMapGraph::new();

    for manifest in index.manifests() {
        add_manifest_to_graph(&mut graph, manifest);
    }

    graph
}

pub fn build_codemap_graph_from_manifests(
    manifests: impl IntoIterator<Item = PluginManifest>,
) -> CodeMapGraph {
    let index = PluginIndex::from_manifests(manifests);
    build_codemap_graph_from_index(&index)
}

fn add_manifest_to_graph(graph: &mut CodeMapGraph, manifest: &PluginManifest) {
    let plugin_node = CodeMapNodeId::Plugin(manifest.id.clone());
    let domain_node = CodeMapNodeId::Domain(DomainId(manifest.family.0.clone()));

    graph.add_node(CodeMapNode::new(
        plugin_node.clone(),
        CodeMapNodeKind::Plugin,
        manifest.id.0.clone(),
    ));

    graph.add_node(CodeMapNode::new(
        domain_node.clone(),
        CodeMapNodeKind::Domain,
        manifest.family.0.clone(),
    ));

    graph.add_edge(CodeMapEdge::new(
        domain_node,
        plugin_node.clone(),
        CodeMapEdgeKind::Owns,
    ));

    add_capabilities(graph, manifest, &plugin_node);
    add_slots(graph, manifest, &plugin_node);
    add_targets(graph, manifest, &plugin_node);
    add_contributions(graph, manifest, &plugin_node);
    add_diagnostics(graph, manifest, &plugin_node);
    add_docs_and_tests(graph, manifest, &plugin_node);
}

fn add_capabilities(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for capability in &manifest.capabilities.provides {
        let node = CodeMapNodeId::Capability(CapabilityId(capability.id.0.clone()));

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::Capability,
            format!("{}@{}", capability.id.0, capability.version),
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::Provides,
        ));
    }

    for capability in &manifest.capabilities.requires {
        let node = CodeMapNodeId::Capability(CapabilityId(capability.id.0.clone()));

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::Capability,
            format!("{}@{}", capability.id.0, capability.version),
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::Requires,
        ));
    }
}

fn add_slots(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for slot in &manifest.slots.implements {
        let node = CodeMapNodeId::Slot(SlotId(slot.0.clone()));

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::Slot,
            slot.0.clone(),
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::ImplementsSlot,
        ));
    }

    for slot in &manifest.slots.requires {
        let node = CodeMapNodeId::Slot(SlotId(slot.0.clone()));

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::Slot,
            slot.0.clone(),
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::Requires,
        ));
    }

    for replaced_plugin in &manifest.slots.replaces {
        let node = CodeMapNodeId::Plugin(replaced_plugin.clone());

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::Plugin,
            replaced_plugin.0.clone(),
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::Replaces,
        ));
    }
}

fn add_targets(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for target in &manifest.targets.reads {
        add_target_edge(graph, plugin_node, target, CodeMapEdgeKind::ReadsTarget);
    }

    for target in &manifest.targets.writes {
        add_target_edge(graph, plugin_node, target, CodeMapEdgeKind::WritesTarget);
    }

    for target in &manifest.targets.contributes {
        add_target_edge(
            graph,
            plugin_node,
            target,
            CodeMapEdgeKind::ContributesTarget,
        );
    }
}

fn add_target_edge(
    graph: &mut CodeMapGraph,
    plugin_node: &CodeMapNodeId,
    target: &TargetId,
    kind: CodeMapEdgeKind,
) {
    let node = CodeMapNodeId::Target(TargetId(target.0.clone()));

    graph.add_node(CodeMapNode::new(
        node.clone(),
        CodeMapNodeKind::Target,
        target.0.clone(),
    ));

    graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, kind));
}

fn add_contributions(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for contribution in &manifest.contributions.emits {
        let label = format!(
            "{}::{}",
            contribution.domain.0, contribution.contribution_type
        );
        let node = CodeMapNodeId::Contribution(label.clone());

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::Contribution,
            label,
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::EmitsContribution,
        ));
    }

    for contribution in &manifest.contributions.consumes {
        let label = format!(
            "{}::{}",
            contribution.domain.0, contribution.contribution_type
        );
        let node = CodeMapNodeId::Contribution(label.clone());

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::Contribution,
            label,
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::ConsumesContribution,
        ));
    }
}

fn add_diagnostics(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for diagnostic in &manifest.diagnostics.channels {
        let node = CodeMapNodeId::DiagnosticChannel(DiagnosticChannelId(
            diagnostic.id.0.clone(),
        ));

        graph.add_node(CodeMapNode::new(
            node.clone(),
            CodeMapNodeKind::DiagnosticChannel,
            diagnostic.id.0.clone(),
        ));

        graph.add_edge(CodeMapEdge::new(
            plugin_node.clone(),
            node,
            CodeMapEdgeKind::ProducesDiagnostic,
        ));
    }
}

fn add_docs_and_tests(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    if let Some(path) = &manifest.docs.pipeline {
        add_doc_edge(graph, plugin_node, path);
    }

    if let Some(path) = &manifest.docs.contributions {
        add_doc_edge(graph, plugin_node, path);
    }

    if let Some(path) = &manifest.docs.diagnostics {
        add_doc_edge(graph, plugin_node, path);
    }

    if let Some(path) = &manifest.tests.hydration {
        add_test_edge(graph, plugin_node, path);
    }

    if let Some(path) = &manifest.tests.participation {
        add_test_edge(graph, plugin_node, path);
    }

    if let Some(path) = &manifest.tests.candidate {
        add_test_edge(graph, plugin_node, path);
    }

    if let Some(path) = &manifest.tests.waterfall {
        add_test_edge(graph, plugin_node, path);
    }

    if let Some(path) = &manifest.tests.diagnostics {
        add_test_edge(graph, plugin_node, path);
    }
}

fn add_doc_edge(graph: &mut CodeMapGraph, plugin_node: &CodeMapNodeId, path: &str) {
    let node = CodeMapNodeId::Doc(path.to_string());

    graph.add_node(
        CodeMapNode::new(node.clone(), CodeMapNodeKind::Doc, path).with_path(path),
    );

    graph.add_edge(CodeMapEdge::new(
        plugin_node.clone(),
        node,
        CodeMapEdgeKind::DocumentedBy,
    ));
}

fn add_test_edge(graph: &mut CodeMapGraph, plugin_node: &CodeMapNodeId, path: &str) {
    let node = CodeMapNodeId::Test(path.to_string());

    graph.add_node(
        CodeMapNode::new(node.clone(), CodeMapNodeKind::Test, path).with_path(path),
    );

    graph.add_edge(CodeMapEdge::new(
        plugin_node.clone(),
        node,
        CodeMapEdgeKind::CoveredByTest,
    ));
}
