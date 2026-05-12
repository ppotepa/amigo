use super::registry::ConsoleCommandRegistry;

pub(crate) mod assets;
mod composition;
mod core;
mod debug;
mod layered;
mod lighting;
mod particles;
mod postfx;
mod render;
mod scene;
mod scheduler;

pub(crate) fn register_builtin_console_commands(registry: &ConsoleCommandRegistry) {
    registry.register(core::CoreConsoleCommandHandler);
    debug::register_debug_console_commands(registry);
    registry.register(scene::SceneConsoleCommandHandler);
    registry.register(assets::AssetsConsoleCommandHandler);
    registry.register(render::RenderConsoleCommandHandler);
    registry.register(composition::Composition2dConsoleCommandHandler);
    registry.register(layered::LayeredImageConsoleCommandHandler);
    registry.register(lighting::Lighting2dConsoleCommandHandler);
    registry.register(particles::ParticlesConsoleCommandHandler);
    registry.register(postfx::PostFxConsoleCommandHandler);
    registry.register(scheduler::SchedulerConsoleCommandHandler);
}
