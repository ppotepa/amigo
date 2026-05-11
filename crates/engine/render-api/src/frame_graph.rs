use crate::composition::{PostFxPassKind, RenderPassInput, RenderPassOutput};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameGraphNodeKind {
    World2D,
    World3D,
    PostFx(PostFxPassKind),
    GameUi,
    DebugOverlay,
    Present,
    LegacyComposite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameGraphNode {
    pub label: String,
    pub kind: FrameGraphNodeKind,
    pub reads: Vec<FrameResourceId>,
    pub writes: Vec<FrameResourceId>,
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
