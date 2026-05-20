use crate::{
    UiDrawPrimitive, UiLayoutNode, UiOverlayDocument, UiOverlayNode, UiOverlayNodeKind,
    UiOverlayTab, UiOverlayViewportScaling, UiRect, UiViewportSize,
    helpers::kind_slug,
    primitives::{append_layout_popup_primitives, append_layout_primitives},
};

include!("layout/entry.rs");
include!("layout/adapter.rs");
include!("layout/tabs.rs");
