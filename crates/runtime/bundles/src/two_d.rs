use amigo_2d_composition::Composition2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_beacon_light_2d_plugin::Beacon2dPlugin;
use amigo_composite_plugin::PostFx2dPlugin;
use amigo_core::AmigoResult;
use amigo_focus_depth_plugin::{DepthMap2dPlugin, FocusTargets2dRuntimePlugin};
use amigo_layered_image_2d_plugin::{
    render::LAYERED_IMAGE_2D_EXTRACTOR_ID, LayeredImagePlugin,
};
use amigo_light_2d_plugin::Lighting2dPlugin;
use amigo_particles_2d_plugin::Particle2dPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder, RuntimePlugin, ServiceRegistry};
use amigo_session::RuntimeSession;
use amigo_shutter_motion_plugin::MOTION_2D_PLUGIN;
use amigo_sprite_2d_plugin::{render::SPRITE_2D_EXTRACTOR_ID, SpritePlugin};
use amigo_text_2d_plugin::{render::TEXT_2D_EXTRACTOR_ID, Text2dPlugin};
use amigo_tilemap_2d_plugin::{render::TILEMAP_2D_EXTRACTOR_ID, TileMap2dPlugin};
use amigo_ui::UiPlugin;
use amigo_vector_2d_plugin::{render::VECTOR_2D_EXTRACTOR_ID, Vector2dPlugin};

use crate::render_extractor_bridges;
use crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry;

const DEPTH_MAP_2D_EXTRACTOR_ID: &str = "depth_map_2d";
const BEACON_2D_EXTRACTOR_ID: &str = "beacon_2d";
const LIGHTING_2D_EXTRACTOR_ID: &str = "lighting_2d";
const COMPOSITION_2D_EXTRACTOR_ID: &str = "composition_2d";
const PARTICLES_2D_EXTRACTOR_ID: &str = "particles_2d";

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

struct WgpuTwoDRenderExtractorBridgePlugin;

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
            .with_plugin(FocusTargets2dRuntimePlugin)?
            .with_plugin(WgpuTwoDRenderExtractorBridgePlugin)
    }
}

impl RuntimePlugin for WgpuTwoDRenderExtractorBridgePlugin {
    fn name(&self) -> &'static str {
        "amigo-wgpu-2d-render-extractor-bridges"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let bridges = registry.required::<WgpuRenderExtractorBridgeRegistry>()?;

        bridges.register(
            SPRITE_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_sprite_extractor,
        );
        bridges.register(
            TEXT_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_text_extractor,
        );
        bridges.register(
            VECTOR_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_vector_extractor,
        );
        bridges.register(
            LAYERED_IMAGE_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_layered_image_extractor,
        );
        bridges.register(
            TILEMAP_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_tilemap_extractor,
        );
        bridges.register(
            DEPTH_MAP_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_depth_map_extractor,
        );
        bridges.register(
            BEACON_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_beacon_extractor,
        );
        bridges.register(
            LIGHTING_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_lighting_extractor,
        );
        bridges.register(
            COMPOSITION_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_composition_extractor,
        );
        bridges.register(
            PARTICLES_2D_EXTRACTOR_ID,
            render_extractor_bridges::register_world_2d_plugin_particles_extractor,
        );

        Ok(())
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
