use amigo_2d_composition::Composition2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_beacon_light_2d_plugin::Beacon2dPlugin;
use amigo_composite_plugin::PostFx2dPlugin;
use amigo_core::AmigoResult;
use amigo_focus_depth_plugin::{DepthMap2dPlugin, FocusTargets2dRuntimePlugin};
use amigo_layered_image_2d_plugin::LayeredImagePlugin;
use amigo_light_2d_plugin::Lighting2dPlugin;
use amigo_particles_2d_plugin::Particle2dPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_session::RuntimeSession;
use amigo_shutter_motion_plugin::MOTION_2D_PLUGIN;
use amigo_sprite_2d_plugin::SpritePlugin;
use amigo_text_2d_plugin::Text2dPlugin;
use amigo_tilemap_2d_plugin::TileMap2dPlugin;
use amigo_ui::UiPlugin;
use amigo_vector_2d_plugin::Vector2dPlugin;

pub use amigo_shutter_motion_plugin::CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL;
pub use amigo_particles_2d_plugin::{
    Particle2dEmitterRuntimeInput, Particle2dSceneService, ParticleAlignMode2d,
    ParticleBlendMode2d, ParticleEmitter2d, ParticleEmitter2dCommand, ParticleLineAnchor2d,
    ParticleMaterial2d, ParticlePreset2d, ParticlePreset2dService, ParticleShape2d,
    ParticleSimulationSpace2d, ParticleSpawnArea2d, ParticleVelocityMode2d,
    tick_particles_2d_world,
};
pub use amigo_ui::{
    UiDocument, UiInputService, UiStateService, collect_scene_ui_font_asset_keys,
    handle_ui_script_command, process_ui_input, resolve_ui_overlay_documents,
    scene_ui_document_to_runtime_document, tick_ui_bindings, UiDrawCommand, UiInputViewportState,
    UiLayer, UiNode, UiNodeKind, UiSceneService, UiScriptCommandContext, UiStyle, UiTarget,
    UiTheme, UiThemePalette, UiThemeService,
};
pub use amigo_layered_image_2d_plugin::{
    LayeredImageScriptCommandContext, LayeredImageScriptCommandOutcome,
    can_handle_layered_image_script_command, handle_layered_image_script_command,
};

pub struct TwoDRuntimeBundle;

impl PluginBundle for TwoDRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-2d-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(SpritePlugin)?
            .with_plugin(LayeredImagePlugin)?
            .with_plugin(DepthMap2dPlugin)?
            .with_plugin(Lighting2dPlugin)?
            .with_plugin(Composition2dPlugin)?
            .with_plugin(PostFx2dPlugin)?
            .with_plugin(Text2dPlugin)?
            .with_plugin(Vector2dPlugin)?
            .with_plugin(Beacon2dPlugin)?
            .with_plugin(Particle2dPlugin)?
            .with_plugin(UiPlugin)?
            .with_plugin(Physics2dPlugin)?
            .with_plugin(TileMap2dPlugin)?
            .with_plugin(MOTION_2D_PLUGIN)?
            .with_plugin(FocusTargets2dRuntimePlugin)
    }
}

pub fn register_two_d_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_text_2d_plugin::register_text2d_runtime_capabilities(session);
    amigo_sprite_2d_plugin::register_sprite2d_runtime_capabilities(session);
    amigo_tilemap_2d_plugin::register_tilemap2d_runtime_capabilities(session);
    amigo_layered_image_2d_plugin::register_layered_image_runtime_capabilities(session);
    amigo_focus_depth_plugin::register_depth_map_runtime_capabilities(session);
    amigo_2d_composition::register_composition2d_runtime_capabilities(session);
    amigo_light_2d_plugin::register_lighting2d_runtime_capabilities(session);
    amigo_composite_plugin::register_post_fx_runtime_capabilities(session);
    amigo_particles_2d_plugin::register_particles2d_runtime_capabilities(session);
    amigo_shutter_motion_plugin::register_motion2d_runtime_capabilities(session);
    amigo_2d_physics::register_physics2d_runtime_capabilities(session);
    amigo_vector_2d_plugin::register_vector2d_runtime_capabilities(session);
    amigo_beacon_light_2d_plugin::register_beacon2d_runtime_capabilities(session);
    amigo_ui::register_ui_runtime_capabilities(session);
}
