use amigo_2d_composition::Composition2dPlugin;
use amigo_layered_image_2d_plugin::LayeredImagePlugin;
use amigo_light_2d_plugin::Lighting2dPlugin;
use amigo_beacon_light_2d_plugin::Beacon2dPlugin;
use amigo_particles_2d_plugin::Particle2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_composite_plugin::PostFx2dPlugin;
use amigo_sprite_2d_plugin::SpritePlugin;
use amigo_text_2d_plugin::Text2dPlugin;
use amigo_tilemap_2d_plugin::TileMap2dPlugin;
use amigo_vector_2d_plugin::Vector2dPlugin;
use amigo_core::AmigoResult;
use amigo_focus_depth_plugin::{DepthMap2dPlugin, FocusTargets2dRuntimePlugin};
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_session::RuntimeSession;
use amigo_shutter_motion_plugin::MOTION_2D_PLUGIN;
use amigo_ui::UiPlugin;

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
