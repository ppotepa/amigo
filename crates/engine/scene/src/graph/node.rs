use super::{SceneGraphNodeId, SceneGraphSemantics};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneGraphNodeKind {
    Root,
    Scene2d,
    Settings,
    Visual2d,
    Objects,
    SceneObject,
    Components,
    Component,
    DrawLayers,
    DrawLayer,
    PostFxHost,
    FramePostFxHost,
    PostFxItem,
    LightGroups,
    LightGroup,
    LightRoutes,
    LightRoute,
    ImagePart,
    Resources,
    Curve2d,
    AssetProxy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringSource {
    pub source_file: String,
    pub yaml_pointer: String,
}

impl AuthoringSource {
    pub fn new(source_file: impl Into<String>, yaml_pointer: impl Into<String>) -> Self {
        Self {
            source_file: source_file.into(),
            yaml_pointer: yaml_pointer.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneGraphNode {
    pub id: SceneGraphNodeId,
    pub label: String,
    pub kind: SceneGraphNodeKind,
    pub source: Option<AuthoringSource>,
    pub parent: Option<SceneGraphNodeId>,
    pub children: Vec<SceneGraphNodeId>,
    pub semantics: SceneGraphSemantics,
}

impl SceneGraphNode {
    pub fn new(
        id: impl Into<SceneGraphNodeId>,
        label: impl Into<String>,
        kind: SceneGraphNodeKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            source: None,
            parent: None,
            children: Vec::new(),
            semantics: SceneGraphSemantics::default(),
        }
    }

    pub fn with_source(mut self, source: AuthoringSource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_parent(mut self, parent: SceneGraphNodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_semantics(mut self, semantics: SceneGraphSemantics) -> Self {
        self.semantics = semantics;
        self
    }
}
