use amigo_core::AmigoResult;
use amigo_runtime::Runtime;

use crate::{
    amigo_3d_material::MaterialSceneService, amigo_3d_mesh::MeshSceneService,
    amigo_3d_text::Text3dSceneService,
    amigo_audio_api::{AudioCommand, AudioSceneService, AudioStateService},
    amigo_audio_mixer::AudioMixerService, amigo_audio_output::AudioOutputBackendService,
    amigo_shutter_motion_plugin::motion_runtime_plugin_report_label,
    amigo_sprite_2d_plugin::SpriteSceneService, amigo_text_2d_plugin::Text2dSceneService,
    amigo_ui::UiSceneService, amigo_vector_2d_plugin::VectorSceneService,
};

#[derive(Debug, Clone)]
pub struct RuntimeBundleSummary {
    pub sprite_entities_2d: Vec<String>,
    pub text_entities_2d: Vec<String>,
    pub vector_entities_2d: Vec<String>,
    pub mesh_entities_3d: Vec<String>,
    pub material_entities_3d: Vec<String>,
    pub text_entities_3d: Vec<String>,
    pub ui_entities: Vec<String>,
    pub audio_clips: Vec<String>,
    pub audio_sources: Vec<String>,
    pub pending_audio_runtime_commands: Vec<String>,
    pub audio_master_volume: f32,
    pub mixed_audio_frame_count: usize,
    pub active_realtime_audio_sources: Vec<String>,
    pub audio_output_started: bool,
    pub audio_output_device: Option<String>,
    pub audio_output_buffered_samples: usize,
    pub audio_output_last_error: Option<String>,
}

pub fn runtime_bundle_summary(runtime: &Runtime) -> AmigoResult<RuntimeBundleSummary> {
    let audio_scene = runtime.required::<AudioSceneService>()?;
    let audio_state = runtime.required::<AudioStateService>()?;
    let audio_mixer = runtime.required::<AudioMixerService>()?;
    let audio_output = runtime.required::<AudioOutputBackendService>()?;
    let sprite_scene = runtime.required::<SpriteSceneService>()?;
    let text_scene = runtime.required::<Text2dSceneService>()?;
    let vector_scene = runtime.required::<VectorSceneService>()?;
    let mesh_scene = runtime.required::<MeshSceneService>()?;
    let text3d_scene = runtime.required::<Text3dSceneService>()?;
    let material_scene = runtime.required::<MaterialSceneService>()?;
    let ui_scene = runtime.required::<UiSceneService>()?;
    let audio_output_snapshot = audio_output.snapshot();

    Ok(RuntimeBundleSummary {
        sprite_entities_2d: sprite_scene.entity_names(),
        text_entities_2d: text_scene.entity_names(),
        vector_entities_2d: vector_scene.entity_names(),
        mesh_entities_3d: mesh_scene.entity_names(),
        material_entities_3d: material_scene.entity_names(),
        text_entities_3d: text3d_scene.entity_names(),
        ui_entities: ui_scene.entity_names(),
        audio_clips: audio_scene
            .clips()
            .into_iter()
            .map(|clip| format!("{} ({:?})", clip.key.as_str(), clip.mode))
            .collect(),
        audio_sources: audio_state
            .playing_sources()
            .into_iter()
            .map(|(source, clip)| format!("{source} -> {}", clip.as_str()))
            .collect(),
        pending_audio_runtime_commands: audio_state
            .pending_runtime_commands()
            .into_iter()
            .map(|command| format_audio_command(&command))
            .collect(),
        audio_master_volume: audio_state.master_volume(),
        mixed_audio_frame_count: audio_mixer.frames().len(),
        active_realtime_audio_sources: audio_mixer.active_realtime_sources(),
        audio_output_started: audio_output_snapshot.started,
        audio_output_device: audio_output_snapshot.device_name,
        audio_output_buffered_samples: audio_output_snapshot.buffered_samples,
        audio_output_last_error: audio_output_snapshot.last_error,
    })
}

pub fn runtime_bundle_plugin_report_label(plugin_name: &str) -> String {
    motion_runtime_plugin_report_label(plugin_name)
}

pub fn format_audio_command(command: &AudioCommand) -> String {
    match command {
        AudioCommand::PlayOnce { clip } => format!("audio.play({})", clip.as_str()),
        AudioCommand::StartSource { source, clip } => {
            format!("audio.start({}, {})", source.as_str(), clip.as_str())
        }
        AudioCommand::StopSource { source } => format!("audio.stop({})", source.as_str()),
        AudioCommand::SetParam {
            source,
            param,
            value,
        } => format!("audio.set_param({}, {}, {})", source.as_str(), param, value),
        AudioCommand::SetVolume { bus, value } => {
            format!("audio.set_volume({}, {})", bus, value)
        }
        AudioCommand::SetMasterVolume { value } => format!("audio.set_master_volume({value})"),
    }
}
