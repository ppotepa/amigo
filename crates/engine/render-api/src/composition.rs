use std::fmt;

use amigo_composite_plugin::{PostFx2dId, PostFxHost2dId, PostFxPipelineKind, PostFxScope2d};

use crate::{
    BlendMode, CameraBinding, ClearMode, CompositionLayer, DepthMode, RenderLayerId, RenderSpace,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderFeatureId(pub String);

impl RenderFeatureId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RenderFeatureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderViewId(pub String);

impl RenderViewId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn main() -> Self {
        Self("main".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RenderViewId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTargetPlan {
    Surface,
    Offscreen { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCompositionStage {
    BeforePostFx,
    AfterPostFx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCompositionStage {
    AfterAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCompositionPolicy {
    pub game_ui_stage: UiCompositionStage,
    pub debug_stage: DebugCompositionStage,
}

impl Default for UiCompositionPolicy {
    fn default() -> Self {
        Self {
            game_ui_stage: UiCompositionStage::AfterPostFx,
            debug_stage: DebugCompositionStage::AfterAll,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameCompositionPlan {
    pub views: Vec<RenderViewPlan>,
    pub layers: Vec<CompositionLayer>,
}

impl FrameCompositionPlan {
    pub fn single_main_view(passes: Vec<RenderPassPlan>) -> Self {
        Self {
            views: vec![RenderViewPlan {
                id: RenderViewId::main(),
                target: RenderTargetPlan::Surface,
                ui_policy: UiCompositionPolicy::default(),
                passes,
            }],
            layers: default_legacy_layers(),
        }
    }

    pub fn with_layers(mut self, layers: Vec<CompositionLayer>) -> Self {
        self.layers = layers;
        self
    }

    pub fn sorted_layers(&self) -> Vec<&CompositionLayer> {
        let mut layers = self.layers.iter().collect::<Vec<_>>();
        layers.sort_by_key(|layer| layer.order);
        layers
    }

    pub fn layers_for_space(&self, space: RenderSpace) -> Vec<&CompositionLayer> {
        self.sorted_layers()
            .into_iter()
            .filter(|layer| layer.space == space)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.views.iter().all(|view| view.passes.is_empty())
    }

    pub fn has_post_fx(&self) -> bool {
        self.views
            .iter()
            .flat_map(|view| view.passes.iter())
            .any(RenderPassPlan::is_post_fx)
    }
}

pub fn default_legacy_layers() -> Vec<CompositionLayer> {
    vec![
        CompositionLayer {
            id: RenderLayerId::new("world"),
            space: RenderSpace::World2D,
            camera: Some(CameraBinding::main()),
            order: 0,
            target: RenderTargetPlan::Surface,
            clear: ClearMode::ClearColor,
            depth: DepthMode::ReadWrite,
            blend: BlendMode::Opaque,
        },
        CompositionLayer {
            id: RenderLayerId::new("game_ui"),
            space: RenderSpace::Ui,
            camera: None,
            order: 100,
            target: RenderTargetPlan::Surface,
            clear: ClearMode::Preserve,
            depth: DepthMode::None,
            blend: BlendMode::Alpha,
        },
        CompositionLayer {
            id: RenderLayerId::new("debug_overlay"),
            space: RenderSpace::DebugOverlay,
            camera: None,
            order: 200,
            target: RenderTargetPlan::Surface,
            clear: ClearMode::Preserve,
            depth: DepthMode::None,
            blend: BlendMode::Alpha,
        },
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderViewPlan {
    pub id: RenderViewId,
    pub target: RenderTargetPlan,
    pub ui_policy: UiCompositionPolicy,
    pub passes: Vec<RenderPassPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderPassPlan {
    World(WorldPassPlan),
    PostFx(PostFxPassPlan),
    GameUi(UiPassPlan),
    DebugOverlay(DebugOverlayPassPlan),
    Present(PresentPassPlan),
}

impl RenderPassPlan {
    pub fn label(&self) -> String {
        match self {
            Self::World(_) => "world".to_owned(),
            Self::PostFx(pass) => format!(
                "post_fx:{}:{}:{}",
                pass.host_id.as_str(),
                pass.effect_id.as_str(),
                pass.feature_id
            ),
            Self::GameUi(_) => "game_ui".to_owned(),
            Self::DebugOverlay(_) => "debug_overlay".to_owned(),
            Self::Present(_) => "present".to_owned(),
        }
    }

    pub fn is_post_fx(&self) -> bool {
        matches!(self, Self::PostFx(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldPassPlan {
    pub output: RenderPassOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostFxPassPlan {
    pub host_id: PostFxHost2dId,
    pub effect_id: PostFx2dId,
    pub scope: PostFxScope2d,
    pub pipeline: PostFxPipelineKind,
    pub feature_id: RenderFeatureId,
    pub input: RenderPassInput,
    pub output: RenderPassOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiPassPlan {
    pub input: RenderPassInput,
    pub output: RenderPassOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebugOverlayPassPlan {
    pub input: RenderPassInput,
    pub output: RenderPassOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresentPassPlan {
    pub input: RenderPassInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPassInput {
    None,
    Surface,
    WorldColor,
    PostFxColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPassOutput {
    Surface,
    WorldColor,
    PostFxColor,
}

impl RenderPassOutput {
    pub fn into_input(self) -> RenderPassInput {
        match self {
            Self::Surface => RenderPassInput::Surface,
            Self::WorldColor => RenderPassInput::WorldColor,
            Self::PostFxColor => RenderPassInput::PostFxColor,
        }
    }
}
