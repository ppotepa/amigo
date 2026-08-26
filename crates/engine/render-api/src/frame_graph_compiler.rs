use std::collections::{BTreeMap, BTreeSet};

use crate::{FrameGraph, FrameGraphDependency, FrameGraphValidationError, FrameResourceId, FrameResourceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameResourceLifetime {
    pub resource: FrameResourceId,
    pub first_node: usize,
    pub last_node: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransientResourceAllocation {
    pub resource: FrameResourceId,
    pub alias_slot: u32,
    pub first_node: usize,
    pub last_node: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledFrameGraph {
    pub node_order: Vec<usize>,
    pub dependencies: Vec<FrameGraphDependency>,
    pub resource_lifetimes: Vec<FrameResourceLifetime>,
    pub transient_allocations: Vec<TransientResourceAllocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameGraphCompileError {
    Invalid(Vec<FrameGraphValidationError>),
    DependencyCycle { nodes: Vec<usize> },
}

pub fn compile_frame_graph(graph: &FrameGraph) -> Result<CompiledFrameGraph, FrameGraphCompileError> {
    graph.validate().map_err(FrameGraphCompileError::Invalid)?;

    let dependencies = compile_dependencies(graph);
    let node_order = topological_order(graph.nodes.len(), &dependencies)?;
    let resource_lifetimes = resource_lifetimes(graph);
    let transient_allocations = allocate_transients(graph, &resource_lifetimes);

    Ok(CompiledFrameGraph {
        node_order,
        dependencies,
        resource_lifetimes,
        transient_allocations,
    })
}

fn compile_dependencies(graph: &FrameGraph) -> Vec<FrameGraphDependency> {
    let mut last_writer = BTreeMap::<FrameResourceId, usize>::new();
    let mut unique = BTreeSet::<(usize, usize, FrameResourceId)>::new();

    for (consumer, node) in graph.nodes.iter().enumerate() {
        for &resource in &node.reads {
            if let Some(&producer) = last_writer.get(&resource) {
                unique.insert((producer, consumer, resource));
            }
        }
        for &resource in &node.writes {
            if let Some(&producer) = last_writer.get(&resource) {
                unique.insert((producer, consumer, resource));
            }
            last_writer.insert(resource, consumer);
        }
    }

    unique
        .into_iter()
        .map(|(producer_node, consumer_node, resource)| FrameGraphDependency {
            producer_node,
            consumer_node,
            resource,
        })
        .collect()
}

fn topological_order(
    node_count: usize,
    dependencies: &[FrameGraphDependency],
) -> Result<Vec<usize>, FrameGraphCompileError> {
    let mut indegree = vec![0usize; node_count];
    let mut outgoing = vec![BTreeSet::<usize>::new(); node_count];
    for dependency in dependencies {
        if outgoing[dependency.producer_node].insert(dependency.consumer_node) {
            indegree[dependency.consumer_node] += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(node, degree)| (*degree == 0).then_some(node))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(node_count);

    while let Some(node) = ready.pop_first() {
        ordered.push(node);
        for &consumer in &outgoing[node] {
            indegree[consumer] -= 1;
            if indegree[consumer] == 0 { ready.insert(consumer); }
        }
    }

    if ordered.len() != node_count {
        return Err(FrameGraphCompileError::DependencyCycle {
            nodes: indegree
                .into_iter()
                .enumerate()
                .filter_map(|(node, degree)| (degree > 0).then_some(node))
                .collect(),
        });
    }
    Ok(ordered)
}

fn resource_lifetimes(graph: &FrameGraph) -> Vec<FrameResourceLifetime> {
    let mut lifetimes = BTreeMap::<FrameResourceId, (usize, usize)>::new();
    for (node_index, node) in graph.nodes.iter().enumerate() {
        for &resource in node.reads.iter().chain(node.writes.iter()) {
            lifetimes
                .entry(resource)
                .and_modify(|range| range.1 = node_index)
                .or_insert((node_index, node_index));
        }
    }
    lifetimes
        .into_iter()
        .map(|(resource, (first_node, last_node))| FrameResourceLifetime {
            resource,
            first_node,
            last_node,
        })
        .collect()
}

fn allocate_transients(
    graph: &FrameGraph,
    lifetimes: &[FrameResourceLifetime],
) -> Vec<TransientResourceAllocation> {
    #[derive(Clone, Copy)]
    struct Slot { id: u32, width: u32, height: u32, last_node: usize }

    let lifetime_by_id = lifetimes
        .iter()
        .map(|lifetime| (lifetime.resource, *lifetime))
        .collect::<BTreeMap<_, _>>();
    let mut resources = graph
        .resources
        .iter()
        .filter_map(|resource| match resource.kind {
            FrameResourceKind::TextureColor { width, height, transient: true } => {
                lifetime_by_id.get(&resource.id).copied().map(|lifetime| (resource.id, width, height, lifetime))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    resources.sort_by_key(|(_, _, _, lifetime)| lifetime.first_node);

    let mut slots = Vec::<Slot>::new();
    let mut allocations = Vec::new();
    for (resource, width, height, lifetime) in resources {
        let reusable = slots
            .iter_mut()
            .find(|slot| slot.width == width && slot.height == height && slot.last_node < lifetime.first_node);
        let slot_id = if let Some(slot) = reusable {
            slot.last_node = lifetime.last_node;
            slot.id
        } else {
            let id = slots.len() as u32;
            slots.push(Slot { id, width, height, last_node: lifetime.last_node });
            id
        };
        allocations.push(TransientResourceAllocation {
            resource,
            alias_slot: slot_id,
            first_node: lifetime.first_node,
            last_node: lifetime.last_node,
        });
    }
    allocations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameGraphNodeKind, FrameResourceKind};

    #[test]
    fn aliases_non_overlapping_transient_textures() {
        let mut graph = FrameGraph::new();
        let a = graph.add_resource("a", FrameResourceKind::TextureColor { width: 64, height: 64, transient: true });
        let b = graph.add_resource("b", FrameResourceKind::TextureColor { width: 64, height: 64, transient: true });
        graph.add_node("write-a", FrameGraphNodeKind::World, vec![], vec![a]);
        graph.add_node("read-a", FrameGraphNodeKind::GameUi, vec![a], vec![]);
        graph.add_node("write-b", FrameGraphNodeKind::World, vec![], vec![b]);
        graph.add_node("read-b", FrameGraphNodeKind::GameUi, vec![b], vec![]);
        let compiled = compile_frame_graph(&graph).expect("graph compiles");
        assert_eq!(compiled.transient_allocations.len(), 2);
        assert_eq!(compiled.transient_allocations[0].alias_slot, compiled.transient_allocations[1].alias_slot);
    }
}
