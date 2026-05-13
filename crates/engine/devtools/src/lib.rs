mod builder;
mod capabilities;
mod completion;
mod command_runtime;
pub mod commands;
mod console;
mod dev_console_overlay;
mod dev_console_theme;
mod debug_overlay_service;
mod emergency_notice;
mod graph;
mod input_router;
mod model;
mod plugin;
mod registry;
mod script_command;
mod snapshot;
mod theme;
mod editor_capability;

pub use builder::{
    build_debug_overlay_document, DebugOverlayRenderExtractor, DebugOverlayRenderOutput,
};
pub use editor_capability::*;
pub use capabilities::{register_console_command_capabilities, register_devtools_capabilities};
pub use command_runtime::{
    dispatch_console_command, register_runtime_console_command_handler, DevConsoleCommandContext,
    RuntimeConsoleCommandHandler, RuntimeConsoleCommandRegistry,
};
pub use completion::{
    ConsoleCompletionContext, ConsoleCompletionEdit, ConsoleCompletionKind,
    ConsoleCompletionSnapshot, ConsoleCompletionState, ConsoleCompletionSuggestion,
    ConsoleRhaiSymbol, ConsoleRhaiValueKind, accept_completion_tab,
    collect_console_rhai_symbols_from_source, compute_console_completion_from_descriptors,
};
pub use console::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand, parse_console_command,
};
pub use input_router::{
    classify_console_input, looks_like_rhai, should_try_rhai_fallback, ConsoleInputKind,
};
pub use dev_console_overlay::{
    build_dev_console_overlay, build_dev_console_overlay_with_theme,
    DevConsoleOverlayRenderExtractor, DevConsoleOverlayRenderOutput,
};
pub use dev_console_theme::DevConsoleTheme;
pub use debug_overlay_service::DebugOverlayService;
pub use emergency_notice::{EmergencyNotice, EmergencyNoticeLevel, EmergencyNoticeService};
pub use model::{
    DebugOverlayCorner, DebugOverlayLayoutMode, DebugOverlayPanel, DebugOverlaySettings,
};
pub use plugin::DevtoolsPlugin;
pub use registry::{ConsoleCommandRegistry, ConsoleCommandSpec};
pub use script_command::{DebugScriptCommandHandler, DevShellScriptCommandHandler};
pub use snapshot::{
    DebugOverlayAudioSnapshot, DebugOverlayFrameSample, DebugOverlayInputSnapshot,
    DebugOverlayParticleSnapshot, DebugOverlaySnapshot,
};

