use crate::{
    helpers::kind_slug,
    primitives::{append_layout_popup_primitives, append_layout_primitives},
    UiDrawPrimitive, UiLayoutNode, UiOverlayDocument, UiOverlayNode, UiOverlayNodeKind,
    UiOverlayTab, UiOverlayViewportScaling, UiRect, UiViewportSize,
};

include!("layout/entry.rs");
include!("layout/adapter.rs");
include!("layout/tabs.rs");
