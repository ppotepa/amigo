use amigo_2d_composition::Composition2dPlugin;
use amigo_2d_layered_image::LayeredImagePlugin;
use amigo_2d_lighting::Lighting2dPlugin;
use amigo_2d_motion::MOTION_2D_PLUGIN;
use amigo_2d_particles::Particle2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_2d_post_fx::PostFx2dPlugin;
use amigo_2d_sprite::SpritePlugin;
use amigo_2d_text::Text2dPlugin;
use amigo_2d_tilemap::TileMap2dPlugin;
use amigo_2d_vector::Vector2dPlugin;
use amigo_3d_material::MaterialPlugin;
use amigo_3d_mesh::MeshPlugin;
use amigo_3d_text::Text3dPlugin;
use amigo_assets::AssetsPlugin;
use amigo_audio_api::AudioApiPlugin;
use amigo_audio_generated::GeneratedAudioPlugin;
use amigo_audio_mixer::AudioMixerPlugin;
use amigo_audio_output::AudioOutputPlugin;
use amigo_behavior::BehaviorPlugin;
use amigo_core::AmigoResult;
use amigo_event_pipeline::EventPipelinePlugin;
use amigo_file_watch_notify::NotifyFileWatchPlugin;
use amigo_hot_reload::HotReloadPlugin;
use amigo_modding::ModdingPlugin;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_scene::ScenePlugin;
use amigo_scripting_rhai::RhaiScriptingPlugin;
use amigo_state::StatePlugin;
use amigo_ui::UiPlugin;
use amigo_core::LaunchSelection;
use amigo_input_actions::InputActionPlugin;
use amigo_input_winit::WinitInputPlugin;
use amigo_render_wgpu::WgpuRenderPlugin;
use amigo_window_winit::WinitWindowPlugin;

pub struct CoreRuntimeBundle;

impl PluginBundle for CoreRuntimeBundle {
    fn name(&self) -> &'static str { "amigo-core-runtime-bundle" }
    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(AssetsPlugin)?
            .with_plugin(HotReloadPlugin)?
            .with_plugin(NotifyFileWatchPlugin)?
            .with_plugin(ScenePlugin)?
            .with_plugin(StatePlugin)?
            .with_plugin(BehaviorPlugin)?
            .with_plugin(EventPipelinePlugin)
    }
}

pub struct TwoDBundle;
impl PluginBundle for TwoDBundle {
    fn name(&self) -> &'static str { "amigo-2d-bundle" }
    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(SpritePlugin)?
            .with_plugin(LayeredImagePlugin)?
            .with_plugin(Lighting2dPlugin)?
            .with_plugin(Composition2dPlugin)?
            .with_plugin(PostFx2dPlugin)?
            .with_plugin(Text2dPlugin)?
            .with_plugin(Vector2dPlugin)?
            .with_plugin(Particle2dPlugin)?
            .with_plugin(UiPlugin)?
            .with_plugin(Physics2dPlugin)?
            .with_plugin(TileMap2dPlugin)?
            .with_plugin(MOTION_2D_PLUGIN)
    }
}

pub struct AudioBundle;
impl PluginBundle for AudioBundle {
    fn name(&self) -> &'static str { "amigo-audio-bundle" }
    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(AudioApiPlugin)?
            .with_plugin(GeneratedAudioPlugin)?
            .with_plugin(AudioMixerPlugin)?
            .with_plugin(AudioOutputPlugin)
    }
}

pub struct ThreeDBundle;
impl PluginBundle for ThreeDBundle {
    fn name(&self) -> &'static str { "amigo-3d-bundle" }
    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(MeshPlugin)?
            .with_plugin(Text3dPlugin)?
            .with_plugin(MaterialPlugin)
    }
}

pub struct ModdingAndScriptingBundle {
    pub modding_plugin: ModdingPlugin,
}
impl PluginBundle for ModdingAndScriptingBundle {
    fn name(&self) -> &'static str { "amigo-modding-and-scripting-bundle" }
    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(self.modding_plugin)?
            .with_plugin(RhaiScriptingPlugin)
    }
}

pub struct PlatformRuntimeBundle<F>
where
    F: Fn(RuntimeBuilder, LaunchSelection) -> AmigoResult<RuntimeBuilder>,
{
    pub launch_selection: LaunchSelection,
    pub app_host_plugins: F,
}

impl<F> PluginBundle for PlatformRuntimeBundle<F>
where
    F: Fn(RuntimeBuilder, LaunchSelection) -> AmigoResult<RuntimeBuilder>,
{
    fn name(&self) -> &'static str { "amigo-platform-runtime-bundle" }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        let builder = builder
            .with_plugin(WinitWindowPlugin::default())?
            .with_plugin(WinitInputPlugin)?
            .with_plugin(InputActionPlugin)?
            .with_plugin(WgpuRenderPlugin)?;
        (self.app_host_plugins)(builder, self.launch_selection)
    }
}
