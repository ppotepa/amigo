//! WGPU-based renderer and overlay implementation.
//! This crate owns GPU setup, scene rendering, and the immediate UI overlay used by tooling and diagnostics.

/// GPU backend bootstrap, surfaces, and platform-facing WGPU helpers.
mod backend;
mod frame_packet;
mod plugin_pass;
mod renderable_adapter;
mod renderable_adapters;
/// Frame extraction and scene rendering code built on top of WGPU.
mod renderer;
/// Immediate overlay model, layout, and drawing primitives for tools.
mod ui_overlay;

pub use amigo_render_api::{RenderSpace2d, Renderable2dItem};
pub use backend::WgpuRenderBackend;
pub use backend::WgpuRenderPlugin;
pub use backend::{WgpuHeadlessContext, WgpuOffscreenTarget, WgpuSurfaceState};
pub use frame_packet::{WgpuRenderFramePacket, WgpuVisualSourceFlags2d};
pub use plugin_pass::*;
pub(crate) use renderable_adapter::*;
pub(crate) use renderable_adapters::*;
pub use renderer::{
    WgpuEmergencyOverlayLevel, WgpuEmergencyOverlayLine, WgpuFrameRenderRequest,
    WgpuFrameRenderTarget, WgpuGameViewportPlacement, WgpuSceneRenderer, WgpuSurfaceRect,
    WgpuWorld2dRenderInput, WgpuWorld3dRenderInput,
};
pub use ui_overlay::{
    UiDrawPrimitive, UiLayoutNode, UiOverlayCurvePoint, UiOverlayDocument, UiOverlayLayer,
    UiOverlayNode, UiOverlayNodeKind, UiOverlayStyle, UiOverlayTab, UiOverlayTextGlow,
    UiOverlayTextOutline, UiOverlayTextShadow, UiOverlayViewport, UiOverlayViewportScaling, UiRect,
    UiTextAnchor, UiViewportSize, build_ui_layout_tree, build_ui_overlay_primitives,
    tab_view_tab_from_mouse,
};
