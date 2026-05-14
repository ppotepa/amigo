use crate::RuntimeConsoleCommandRegistry as ConsoleCommandRegistry;

mod assets;
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

pub use assets::request_asset_reload;

pub fn register_builtin_console_commands(registry: &ConsoleCommandRegistry) {
    crate::register_runtime_console_command_handler(registry, core::CoreConsoleCommandHandler);
    debug::register_debug_console_commands(registry);
    crate::register_runtime_console_command_handler(registry, scene::SceneConsoleCommandHandler);
    crate::register_runtime_console_command_handler(registry, assets::AssetsConsoleCommandHandler);
    crate::register_runtime_console_command_handler(registry, render::RenderConsoleCommandHandler);
    crate::register_runtime_console_command_handler(
        registry,
        composition::Composition2dConsoleCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        layered::LayeredImageConsoleCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        lighting::Lighting2dConsoleCommandHandler,
    );
    crate::register_runtime_console_command_handler(
        registry,
        particles::ParticlesConsoleCommandHandler,
    );
    crate::register_runtime_console_command_handler(registry, postfx::PostFxConsoleCommandHandler);
    crate::register_runtime_console_command_handler(
        registry,
        scheduler::SchedulerConsoleCommandHandler,
    );
}
