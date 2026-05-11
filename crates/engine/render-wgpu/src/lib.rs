//! WGPU-based renderer and overlay implementation.
//! This crate owns GPU setup, scene rendering, and the immediate UI overlay used by tooling and diagnostics.

/// GPU backend bootstrap, surfaces, and platform-facing WGPU helpers.
mod backend;
/// Frame extraction and scene rendering code built on top of WGPU.
mod renderer;
/// Immediate overlay model, layout, and drawing primitives for tools.
mod ui_overlay;

pub use backend::WgpuRenderBackend;
pub use backend::WgpuRenderPlugin;
pub use backend::{WgpuHeadlessContext, WgpuOffscreenTarget, WgpuSurfaceState};
pub use renderer::{
    WgpuFrameGraphExecutionMode, WgpuFrameRenderRequest, WgpuSceneRenderer,
    WgpuWorld2dRenderInput, WgpuWorld3dRenderInput,
};
pub use ui_overlay::{
    UiDrawPrimitive, UiLayoutNode, UiOverlayCurvePoint, UiOverlayDocument, UiOverlayLayer,
    UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle, UiOverlayTab, UiOverlayViewport,
    UiOverlayViewportScaling, UiRect, UiTextAnchor, UiViewportSize, build_ui_layout_tree,
    build_ui_overlay_primitives, tab_view_tab_from_mouse,
};
