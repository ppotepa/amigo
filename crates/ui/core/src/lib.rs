//! Runtime UI document, layout, and state services.
//! It owns authored UI models plus bindings, themes, and input state consumed by the app and renderer.

/// UI input collection and focus/interaction state helpers.
mod input;
/// Layout measurement and flow logic for authored UI documents.
mod layout;
/// Shared UI document, style, and event data structures.
mod model;
/// Runtime plugin wiring for the UI domain.
mod plugin;
/// Adapters that hydrate scene-authored UI data into runtime state.
mod scene_bridge;
/// Core UI services for documents, bindings, theme, and live state.
mod service;

pub use input::{UiInputService, UiInputSnapshot};
pub use layout::{UiLayoutService, compute_layout, hit_test};
pub use model::{
    UiBinds, UiCurveEdit, UiCurvePoint, UiDocument, UiEventBinding, UiEvents, UiLayer,
    UiLayoutNode, UiNode, UiNodeKind, UiRect, UiStyle, UiTab, UiTarget, UiTextAlign, UiTheme,
    UiThemePalette, UiViewport, UiViewportScaling, curve_editor_edit_from_mouse,
    curve_points_from_values, default_curve_points, format_curve_points, normalize_curve_points,
};
pub use plugin::UiPlugin;
pub use scene_bridge::{collect_scene_ui_font_asset_keys, scene_ui_document_to_runtime_document};
pub use service::{
    UiDomainInfo, UiDrawCommand, UiModelBinding, UiModelBindingKind, UiModelBindingService,
    UiSceneService, UiStateService, UiStateSnapshot, UiThemeService, register_ui_services,
};
