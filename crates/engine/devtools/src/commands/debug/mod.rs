use crate::RuntimeConsoleCommandRegistry as ConsoleCommandRegistry;

mod audio;
mod dump;
mod fps;
mod fps_graph;
mod graphs;
mod input;
mod layers;
mod lights;
mod memory;
mod overlay;
mod overlay_corner;
mod overlay_mode;
mod overlay_scale;
mod particles;
mod render;
mod reset;
mod scheduler;
mod shared;
mod stats;
mod timings;

pub(crate) fn register_debug_console_commands(registry: &ConsoleCommandRegistry) {
    crate::register_runtime_console_command_handler(
        registry,
        overlay::DebugOverlayCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        overlay_mode::DebugOverlayModeCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        overlay_scale::DebugOverlayScaleCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        overlay_corner::DebugOverlayCornerCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        fps::DebugFpsCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        fps_graph::DebugFpsGraphCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        graphs::DebugGraphsCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        stats::DebugStatsCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        particles::DebugParticlesCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        render::DebugRenderCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        audio::DebugAudioCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        input::DebugInputCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        lights::DebugLightsCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        layers::DebugLayersCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        timings::DebugTimingsCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        scheduler::DebugSchedulerCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        memory::DebugMemoryCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        dump::DebugDumpCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        reset::DebugResetCommandHandler,
    );
}



