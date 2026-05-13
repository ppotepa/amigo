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
/// Runtime contribution descriptors for UI-owned scene handlers and systems.
mod runtime_capabilities;
mod runtime_ui;
/// Adapters that hydrate scene-authored UI data into runtime state.
mod scene_bridge;
/// Scene command execution owned by the UI domain.
mod scene_command;
mod script_command;
/// Core UI services for documents, bindings, theme, and live state.
mod service;
mod systems;
mod editor_capability;

pub use input::{UiInputService, UiInputSnapshot, UiInputViewportState};
pub use editor_capability::*;
pub use layout::{UiLayoutService, compute_layout, hit_test};
pub use model::{
    UiBinds, UiCurveEdit, UiCurvePoint, UiDocument, UiEventBinding, UiEvents, UiLayer,
    UiLayoutNode, UiNode, UiNodeKind, UiRect, UiStyle, UiTab, UiTarget, UiTextAlign, UiTheme,
    UiThemePalette, UiViewport, UiViewportScaling, curve_editor_edit_from_mouse,
    curve_points_from_values, default_curve_points, format_curve_points, normalize_curve_points,
};
pub use plugin::UiPlugin;
pub use runtime_capabilities::*;
pub use runtime_ui::{
    dropdown_visible_option_count, find_ui_layout_node, hit_test_ui_layout, process_ui_input,
    resolve_ui_overlay_documents, UiOverlayRenderExtractor, UiOverlayRenderOutput,
};
pub use scene_bridge::{collect_scene_ui_font_asset_keys, scene_ui_document_to_runtime_document};
pub use scene_command::*;
pub use script_command::*;
pub use service::{
    UiDomainInfo, UiDrawCommand, UiModelBinding, UiModelBindingKind, UiModelBindingService,
    UiSceneService, UiStateService, UiStateSnapshot, UiThemeService, register_ui_services,
};
pub use systems::*;

