#[derive(Debug, Clone, PartialEq)]
pub struct UiDocument {
    pub target: UiTarget,
    pub root: UiNode,
}

impl UiDocument {
    pub fn screen_space(layer: UiLayer, root: UiNode) -> Self {
        Self {
            target: UiTarget::ScreenSpace {
                layer,
                viewport: None,
            },
            root,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiTarget {
    ScreenSpace {
        layer: UiLayer,
        viewport: Option<UiViewport>,
    },
}

impl UiTarget {
    pub fn layer(&self) -> UiLayer {
        match self {
            Self::ScreenSpace { layer, .. } => *layer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiViewport {
    pub width: f32,
    pub height: f32,
    pub scaling: UiViewportScaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiViewportScaling {
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiLayer {
    Background,
    Hud,
    Menu,
    Debug,
}

