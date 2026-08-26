use amigo_codemap_api::{
    CodeMapEdge, CodeMapEdgeKind, CodeMapGraph, CodeMapNode, CodeMapNodeId, CodeMapNodeKind,
};
use amigo_plugin_api::{
    CapabilityId, CapabilityRef, DiagnosticChannelId, DomainId, PluginManifest, SlotId, TargetId,
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

fn capability_node_id(capability: &CapabilityRef) -> CodeMapNodeId {
    CodeMapNodeId::Capability(CapabilityId(format!(
        "{}@{}",
        capability.id.0, capability.version
    )))
}

fn add_capabilities(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for capability in &manifest.capabilities.provides {
        let node = capability_node_id(capability);
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
        let node = capability_node_id(capability);
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

fn add_slots(graph: &mut CodeMapGraph, manifest: &PluginManifest, plugin_node: &CodeMapNodeId) {
    for slot in &manifest.slots.implements {
        let node = CodeMapNodeId::Slot(SlotId(slot.0.clone()));
        graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Slot, slot.0.clone()));
        graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::ImplementsSlot));
    }
    for slot in &manifest.slots.requires {
        let node = CodeMapNodeId::Slot(SlotId(slot.0.clone()));
        graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Slot, slot.0.clone()));
        graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::Requires));
    }
    for replaced_plugin in &manifest.slots.replaces {
        let node = CodeMapNodeId::Plugin(replaced_plugin.clone());
        graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Plugin, replaced_plugin.0.clone()));
        graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::Replaces));
    }
}

fn add_targets(graph: &mut CodeMapGraph, manifest: &PluginManifest, plugin_node: &CodeMapNodeId) {
    for target in &manifest.targets.reads {
        add_target_edge(graph, plugin_node, target, CodeMapEdgeKind::ReadsTarget);
    }
    for target in &manifest.targets.writes {
        add_target_edge(graph, plugin_node, target, CodeMapEdgeKind::WritesTarget);
    }
    for target in &manifest.targets.contributes {
        add_target_edge(graph, plugin_node, target, CodeMapEdgeKind::ContributesTarget);
    }
}

fn add_target_edge(
    graph: &mut CodeMapGraph,
    plugin_node: &CodeMapNodeId,
    target: &TargetId,
    kind: CodeMapEdgeKind,
) {
    let node = CodeMapNodeId::Target(TargetId(target.0.clone()));
    graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Target, target.0.clone()));
    graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, kind));
}

fn add_contributions(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for contribution in &manifest.contributions.emits {
        let label = format!("{}::{}", contribution.domain.0, contribution.contribution_type);
        let node = CodeMapNodeId::Contribution(label.clone());
        graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Contribution, label));
        graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::EmitsContribution));
    }
    for contribution in &manifest.contributions.consumes {
        let label = format!("{}::{}", contribution.domain.0, contribution.contribution_type);
        let node = CodeMapNodeId::Contribution(label.clone());
        graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Contribution, label));
        graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::ConsumesContribution));
    }
}

fn add_diagnostics(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for diagnostic in &manifest.diagnostics.channels {
        let node = CodeMapNodeId::DiagnosticChannel(DiagnosticChannelId(diagnostic.id.0.clone()));
        graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::DiagnosticChannel, diagnostic.id.0.clone()));
        graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::ProducesDiagnostic));
    }
}

fn add_docs_and_tests(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
) {
    for path in [
        manifest.docs.pipeline.as_ref(),
        manifest.docs.contributions.as_ref(),
        manifest.docs.diagnostics.as_ref(),
    ].into_iter().flatten() {
        add_doc_edge(graph, manifest, plugin_node, path);
    }
    for path in [
        manifest.tests.hydration.as_ref(),
        manifest.tests.participation.as_ref(),
        manifest.tests.candidate.as_ref(),
        manifest.tests.waterfall.as_ref(),
        manifest.tests.diagnostics.as_ref(),
    ].into_iter().flatten() {
        add_test_edge(graph, manifest, plugin_node, path);
    }
}

fn owned_path_key(manifest: &PluginManifest, path: &str) -> String {
    format!("{}::{path}", manifest.id.0)
}

fn add_doc_edge(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
    path: &str,
) {
    let node = CodeMapNodeId::Doc(owned_path_key(manifest, path));
    graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Doc, path).with_path(path));
    graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::DocumentedBy));
}

fn add_test_edge(
    graph: &mut CodeMapGraph,
    manifest: &PluginManifest,
    plugin_node: &CodeMapNodeId,
    path: &str,
) {
    let node = CodeMapNodeId::Test(owned_path_key(manifest, path));
    graph.add_node(CodeMapNode::new(node.clone(), CodeMapNodeKind::Test, path).with_path(path));
    graph.add_edge(CodeMapEdge::new(plugin_node.clone(), node, CodeMapEdgeKind::CoveredByTest));
}
