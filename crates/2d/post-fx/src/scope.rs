use crate::{PostFx2d, PostFx2dStack};

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

string_id!(PostFxHost2dId);
string_id!(PostFx2dId);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PostFxScope2d {
    Frame,
    DrawLayer {
        draw_layer_id: String,
    },
    SceneObjectPixels {
        scene_object_id: String,
    },
    GroupSubtree {
        root_scene_object_id: String,
    },
    SourceImage {
        asset: String,
    },
    ImagePart {
        owner_scene_object_id: String,
        component_id: Option<String>,
        part_id: String,
    },
}

impl PostFxScope2d {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Frame => "Frame",
            Self::DrawLayer { .. } => "Draw Layer",
            Self::SceneObjectPixels { .. } => "Scene Object Pixels",
            Self::GroupSubtree { .. } => "Group Subtree",
            Self::SourceImage { .. } => "Source Image",
            Self::ImagePart { .. } => "Image Part",
        }
    }

    pub fn default_pipeline(&self) -> PostFxPipelineKind {
        match self {
            Self::Frame => PostFxPipelineKind::FrameGraph,
            Self::DrawLayer { .. } => PostFxPipelineKind::OffscreenDrawLayer,
            Self::SceneObjectPixels { .. } => PostFxPipelineKind::OffscreenObject,
            Self::GroupSubtree { .. } => PostFxPipelineKind::OffscreenGroup,
            Self::SourceImage { .. } | Self::ImagePart { .. } => PostFxPipelineKind::CachedImage,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostFxPipelineKind {
    FrameGraph,
    CachedImage,
    OffscreenObject,
    OffscreenDrawLayer,
    OffscreenGroup,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostFx2dInstance {
    pub id: PostFx2dId,
    pub effect: PostFx2d,
}

impl PostFx2dInstance {
    pub fn new(id: impl Into<PostFx2dId>, effect: PostFx2d) -> Self {
        Self {
            id: id.into(),
            effect,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopedPostFx2dStack {
    pub host_id: PostFxHost2dId,
    pub scope: PostFxScope2d,
    pub pipeline: PostFxPipelineKind,
    pub effects: Vec<PostFx2dInstance>,
}

impl ScopedPostFx2dStack {
    pub fn new(
        host_id: impl Into<PostFxHost2dId>,
        scope: PostFxScope2d,
        effects: Vec<PostFx2dInstance>,
    ) -> Self {
        let pipeline = scope.default_pipeline();
        Self {
            host_id: host_id.into(),
            scope,
            pipeline,
            effects,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn normalized(self) -> Self {
        Self {
            host_id: self.host_id,
            scope: self.scope,
            pipeline: self.pipeline,
            effects: self
                .effects
                .into_iter()
                .map(|instance| PostFx2dInstance {
                    id: instance.id,
                    effect: instance.effect.normalized(),
                })
                .filter(|instance| instance.effect.is_active())
                .collect(),
        }
    }

    pub fn as_frame_stack(&self) -> PostFx2dStack {
        PostFx2dStack {
            effects: self
                .effects
                .iter()
                .map(|instance| instance.effect.clone())
                .collect(),
        }
    }

    pub fn from_frame_stack(stack: PostFx2dStack) -> Self {
        let effects = stack
            .effects
            .into_iter()
            .enumerate()
            .map(|(index, effect)| PostFx2dInstance::new(format!("frame_fx_{index:03}"), effect))
            .collect();

        Self::new("frame", PostFxScope2d::Frame, effects).normalized()
    }

    pub fn push_frame_effect(&mut self, effect: PostFx2d) {
        let index = self.effects.len();
        self.effects.push(PostFx2dInstance::new(
            format!("{}:{index}:frame", self.host_id.as_str()),
            effect,
        ));
    }
}

pub type FramePostFx2dStack = ScopedPostFx2dStack;
pub type DrawLayerPostFx2dStack = ScopedPostFx2dStack;
pub type ObjectPostFx2dStack = ScopedPostFx2dStack;
pub type GroupPostFx2dStack = ScopedPostFx2dStack;
pub type SourceImagePostFx2dStack = ScopedPostFx2dStack;
pub type ImagePartPostFx2dStack = ScopedPostFx2dStack;
