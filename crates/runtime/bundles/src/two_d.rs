use amigo_2d_composition::Composition2dPlugin;
use amigo_2d_depth_map::DepthMap2dPlugin;
use amigo_2d_layered_image::LayeredImagePlugin;
use amigo_2d_lighting::Lighting2dPlugin;
use amigo_2d_lighting_beacon::Beacon2dPlugin;
use amigo_2d_motion::MOTION_2D_PLUGIN;
use amigo_2d_particles::Particle2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_2d_post_fx::PostFx2dPlugin;
use amigo_2d_sprite::SpritePlugin;
use amigo_2d_text::Text2dPlugin;
use amigo_2d_tilemap::TileMap2dPlugin;
use amigo_2d_vector::Vector2dPlugin;
use amigo_core::AmigoResult;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_session::RuntimeSession;
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
            .with_plugin(MOTION_2D_PLUGIN)
    }
}

pub fn register_two_d_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_2d_text::register_text2d_runtime_capabilities(session);
    amigo_2d_sprite::register_sprite2d_runtime_capabilities(session);
    amigo_2d_tilemap::register_tilemap2d_runtime_capabilities(session);
    amigo_2d_layered_image::register_layered_image_runtime_capabilities(session);
    amigo_2d_depth_map::register_depth_map_runtime_capabilities(session);
    amigo_2d_composition::register_composition2d_runtime_capabilities(session);
    amigo_2d_lighting::register_lighting2d_runtime_capabilities(session);
    amigo_2d_post_fx::register_post_fx_runtime_capabilities(session);
    amigo_2d_particles::register_particles2d_runtime_capabilities(session);
    amigo_2d_motion::register_motion2d_runtime_capabilities(session);
    amigo_2d_physics::register_physics2d_runtime_capabilities(session);
    amigo_2d_vector::register_vector2d_runtime_capabilities(session);
    amigo_2d_lighting_beacon::register_beacon2d_runtime_capabilities(session);
    amigo_ui::register_ui_runtime_capabilities(session);
}
