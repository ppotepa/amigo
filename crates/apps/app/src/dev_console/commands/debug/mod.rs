use crate::dev_console::registry::ConsoleCommandRegistry;

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
    registry.register(overlay::DebugOverlayCommandHandler);
    registry.register(overlay_mode::DebugOverlayModeCommandHandler);
    registry.register(overlay_scale::DebugOverlayScaleCommandHandler);
    registry.register(overlay_corner::DebugOverlayCornerCommandHandler);
    registry.register(fps::DebugFpsCommandHandler);
    registry.register(fps_graph::DebugFpsGraphCommandHandler);
    registry.register(graphs::DebugGraphsCommandHandler);
    registry.register(stats::DebugStatsCommandHandler);
    registry.register(particles::DebugParticlesCommandHandler);
    registry.register(render::DebugRenderCommandHandler);
    registry.register(audio::DebugAudioCommandHandler);
    registry.register(input::DebugInputCommandHandler);
    registry.register(lights::DebugLightsCommandHandler);
    registry.register(layers::DebugLayersCommandHandler);
    registry.register(timings::DebugTimingsCommandHandler);
    registry.register(scheduler::DebugSchedulerCommandHandler);
    registry.register(memory::DebugMemoryCommandHandler);
    registry.register(dump::DebugDumpCommandHandler);
    registry.register(reset::DebugResetCommandHandler);
}
