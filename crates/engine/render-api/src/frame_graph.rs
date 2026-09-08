use std::collections::{BTreeMap, BTreeSet};

use crate::composition::{RenderFeatureId, RenderPassInput, RenderPassOutput};
use crate::{PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FrameResourceId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameResourceKind {
    SurfaceColor,
    TextureColor {
        width: u32,
        height: u32,
        transient: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameGraphResource {
    pub id: FrameResourceId,
    pub label: String,
    pub kind: FrameResourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameGraphNodeKind {
    World,
    PostFx {
        host_id: PostFxHost2dId,
        effect_id: PostFx2dId,
        scope: PostFxScope2d,
        pipeline: PostFxPipelineKind,
        feature_id: RenderFeatureId,
    },
    GameUi,
    DebugOverlay,
    Present,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameGraphNode {
    pub label: String,
    pub kind: FrameGraphNodeKind,
    pub reads: Vec<FrameResourceId>,
    pub writes: Vec<FrameResourceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameGraphValidationError {
    MissingResource {
        node: String,
        resource: FrameResourceId,
    },
    ReadBeforeWrite {
        node: String,
        resource: FrameResourceId,
    },
    ExternalTargetWrittenByNonPresent {
        node: String,
        resource: FrameResourceId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameGraphDependency {
    pub producer_node: usize,
    pub consumer_node: usize,
    pub resource: FrameResourceId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrameGraph {
    pub resources: Vec<FrameGraphResource>,
    pub nodes: Vec<FrameGraphNode>,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_resource(
        &mut self,
        label: impl Into<String>,
        kind: FrameResourceKind,
    ) -> FrameResourceId {
        let id = FrameResourceId(self.resources.len() as u32);
        self.resources.push(FrameGraphResource {
            id,
            label: label.into(),
            kind,
        });
        id
    }

    pub fn add_node(
        &mut self,
        label: impl Into<String>,
        kind: FrameGraphNodeKind,
        reads: Vec<FrameResourceId>,
        writes: Vec<FrameResourceId>,
    ) {
        self.nodes.push(FrameGraphNode {
            label: label.into(),
            kind,
            reads,
            writes,
        });
    }

    pub fn node_labels(&self) -> Vec<&str> {
        self.nodes.iter().map(|node| node.label.as_str()).collect()
    }

    pub fn validate(&self) -> Result<(), Vec<FrameGraphValidationError>> {
        let known = self
            .resources
            .iter()
            .map(|resource| resource.id)
            .collect::<BTreeSet<_>>();
        let external = self
            .resources
            .iter()
            .filter(|resource| is_external_resource(&resource.kind))
            .map(|resource| resource.id)
            .collect::<BTreeSet<_>>();
        let mut available = external.clone();
        let mut errors = Vec::new();

        for node in &self.nodes {
            for &read in &node.reads {
                if !known.contains(&read) {
                    errors.push(FrameGraphValidationError::MissingResource {
                        node: node.label.clone(),
                        resource: read,
                    });
                } else if !available.contains(&read) {
                    errors.push(FrameGraphValidationError::ReadBeforeWrite {
                        node: node.label.clone(),
                        resource: read,
                    });
                }
            }

            for &write in &node.writes {
                if !known.contains(&write) {
                    errors.push(FrameGraphValidationError::MissingResource {
                        node: node.label.clone(),
                        resource: write,
                    });
                    continue;
                }
                if external.contains(&write) && !matches!(node.kind, FrameGraphNodeKind::Present) {
                    errors.push(
                        FrameGraphValidationError::ExternalTargetWrittenByNonPresent {
                            node: node.label.clone(),
                            resource: write,
                        },
                    );
                }
                available.insert(write);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn dependencies(&self) -> Vec<FrameGraphDependency> {
        let mut last_writer = BTreeMap::<FrameResourceId, usize>::new();
        let mut dependencies = Vec::new();
        for (consumer_node, node) in self.nodes.iter().enumerate() {
            for &resource in &node.reads {
                if let Some(&producer_node) = last_writer.get(&resource) {
                    dependencies.push(FrameGraphDependency {
                        producer_node,
                        consumer_node,
                        resource,
                    });
                }
            }
            for &resource in &node.writes {
                last_writer.insert(resource, consumer_node);
            }
        }
        dependencies
    }
}

fn is_external_resource(kind: &FrameResourceKind) -> bool {
    matches!(kind, FrameResourceKind::SurfaceColor)
        || matches!(
            kind,
            FrameResourceKind::TextureColor {
                transient: false,
                ..
            }
        )
}

pub fn resource_for_input(
    input: RenderPassInput,
    surface: FrameResourceId,
    world: FrameResourceId,
    post_fx: FrameResourceId,
) -> Option<FrameResourceId> {
    match input {
        RenderPassInput::None => None,
        RenderPassInput::Surface => Some(surface),
        RenderPassInput::WorldColor => Some(world),
        RenderPassInput::PostFxColor => Some(post_fx),
    }
}

pub fn resource_for_output(
    output: RenderPassOutput,
    surface: FrameResourceId,
    world: FrameResourceId,
    post_fx: FrameResourceId,
) -> FrameResourceId {
    match output {
        RenderPassOutput::Surface => surface,
        RenderPassOutput::WorldColor => world,
        RenderPassOutput::PostFxColor => post_fx,
    }
}
