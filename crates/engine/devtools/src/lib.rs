mod builder;
mod capabilities;
mod command_runtime;
pub mod commands;
mod completion;
mod console;
mod console_input_controller;
mod debug_overlay_service;
mod dev_console_overlay;
mod dev_console_theme;
mod editor_capability;
mod emergency_notice;
mod graph;
mod input_router;
mod model;
mod plugin;
mod plugin_diagnostics;
mod registry;
mod script_command;
mod snapshot;
mod theme;

pub use builder::{
    DebugOverlayRenderExtractor, DebugOverlayRenderOutput, build_debug_overlay_document,
};
pub use capabilities::{register_console_command_capabilities, register_devtools_capabilities};
pub use command_runtime::{
    DevConsoleCommandContext, RuntimeConsoleCommandHandler, RuntimeConsoleCommandRegistry,
    dispatch_console_command, register_runtime_console_command_handler,
};
pub use completion::{
    ConsoleCompletionContext, ConsoleCompletionEdit, ConsoleCompletionKind,
    ConsoleCompletionSnapshot, ConsoleCompletionState, ConsoleCompletionSuggestion,
    ConsoleRhaiSymbol, ConsoleRhaiValueKind, accept_completion_tab,
    collect_console_rhai_symbols_from_source, compute_console_completion_from_descriptors,
};
pub use console::{
    ConsoleArgKind, ConsoleArgSpec, ConsoleCommandDescriptor, ConsoleCommandForm,
    ConsoleCommandResult, ConsoleCommandSchema, ParsedConsoleCommand, parse_console_command,
};
pub use console_input_controller::{DevConsoleInputController, DevConsoleInputOutcome};
pub use debug_overlay_service::DebugOverlayService;
pub use dev_console_overlay::{
    DevConsoleOverlayRenderExtractor, DevConsoleOverlayRenderOutput, build_dev_console_overlay,
    build_dev_console_overlay_with_theme,
};
pub use dev_console_theme::DevConsoleTheme;
pub use editor_capability::*;
pub use emergency_notice::{EmergencyNotice, EmergencyNoticeLevel, EmergencyNoticeService};
pub use input_router::{
    ConsoleInputKind, ConsoleInputRoute, RoutedConsoleInput, classify_console_input,
    looks_like_rhai, route_console_input, should_try_rhai_fallback,
};
pub use model::{
    DebugOverlayCorner, DebugOverlayLayoutMode, DebugOverlayPanel, DebugOverlaySettings,
};
pub use plugin::DevtoolsPlugin;
pub use plugin_diagnostics::*;
pub use registry::{ConsoleCommandRegistry, ConsoleCommandSpec};
pub use script_command::{DebugScriptCommandHandler, DevShellScriptCommandHandler};
pub use snapshot::{
    DebugOverlayAudioSnapshot, DebugOverlayFrameSample, DebugOverlayInputSnapshot,
    DebugOverlayParticleSnapshot, DebugOverlaySnapshot,
};
