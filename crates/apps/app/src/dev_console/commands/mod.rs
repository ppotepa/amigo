use super::registry::ConsoleCommandRegistry;

pub(crate) mod assets;
mod composition;
mod core;
mod layered;
mod lighting;
mod particles;
mod render;
mod scene;

pub(crate) fn register_builtin_console_commands(registry: &ConsoleCommandRegistry) {
    registry.register(core::CoreConsoleCommandHandler);
    registry.register(scene::SceneConsoleCommandHandler);
    registry.register(assets::AssetsConsoleCommandHandler);
    registry.register(render::RenderConsoleCommandHandler);
    registry.register(particles::ParticlesConsoleCommandHandler);
    registry.register(layered::LayeredImageConsoleCommandHandler);
    registry.register(lighting::Lighting2dConsoleCommandHandler);
    registry.register(composition::Composition2dConsoleCommandHandler);
}
