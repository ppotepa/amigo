use super::registry::ConsoleCommandRegistry;

pub(crate) mod assets;
mod core;
mod debug;
mod render;
mod scene;
mod scheduler;

pub(crate) fn register_builtin_console_commands(registry: &ConsoleCommandRegistry) {
    registry.register(core::CoreConsoleCommandHandler);
    debug::register_debug_console_commands(registry);
    registry.register(scene::SceneConsoleCommandHandler);
    registry.register(assets::AssetsConsoleCommandHandler);
    registry.register(render::RenderConsoleCommandHandler);
    registry.register(scheduler::SchedulerConsoleCommandHandler);
}
