use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
};
use amigo_core::AmigoResult;
use amigo_event_pipeline::{EventPipelineService, EventPipelineStep};
use amigo_particles_2d_plugin::Particle2dSceneService;
use amigo_runtime::Runtime;
use amigo_scene::{SceneCommand, SceneCommandQueue, SceneKey};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue, ScriptRuntimeService};
use amigo_state::SceneStateService;
use amigo_ui::UiStateService;

use crate::{
    AudioClip, AudioClipKey, AudioCommand, AudioCommandQueue, AudioPlaybackMode, AudioSceneService,
};

pub fn run_event_pipelines_for_event(
    runtime: &Runtime,
    event: &ScriptEvent,
    resolve_audio_clip: impl Fn(&str) -> AssetKey,
    mut run_script: impl FnMut(&ScriptRuntimeService, &str, &ScriptEvent) -> AmigoResult<()>,
) -> AmigoResult<()> {
    let pipelines = runtime.required::<EventPipelineService>()?;
    let state = runtime.required::<SceneStateService>()?;
    let ui = runtime.resolve::<UiStateService>();
    let particles = runtime.resolve::<Particle2dSceneService>();
    let audio_commands = runtime.resolve::<AudioCommandQueue>();
    let scene_commands = runtime.resolve::<SceneCommandQueue>();
    let script_events = runtime.resolve::<ScriptEventQueue>();
    let script_runtime = runtime.resolve::<ScriptRuntimeService>();
    let asset_catalog = runtime.resolve::<AssetCatalog>();
    let audio_scene = runtime.resolve::<AudioSceneService>();

    for pipeline in pipelines.pipelines_for_topic(&event.topic) {
        for step in pipeline.steps {
            match step {
                EventPipelineStep::PlayAudio { clip } => {
                    if let Some(audio_commands) = audio_commands.as_ref() {
                        let asset_key = resolve_audio_clip(&clip);
                        if let (Some(asset_catalog), Some(audio_scene)) =
                            (asset_catalog.as_ref(), audio_scene.as_ref())
                        {
                            register_audio_clip_reference(
                                asset_catalog.as_ref(),
                                audio_scene.as_ref(),
                                &asset_key,
                                AudioPlaybackMode::OneShot,
                            );
                        }
                        audio_commands.push(AudioCommand::PlayOnce {
                            clip: AudioClipKey::new(asset_key.as_str()),
                        });
                    }
                }
                EventPipelineStep::SetState { key, value } => {
                    set_state_from_string(state.as_ref(), key, value);
                }
                EventPipelineStep::IncrementState { key, by } => {
                    let current = state.get_float(&key).unwrap_or(0.0);
                    state.set_float(key, current + by);
                }
                EventPipelineStep::ShowUi { path } => {
                    if let Some(ui) = ui.as_ref() {
                        ui.show(path);
                    }
                }
                EventPipelineStep::HideUi { path } => {
                    if let Some(ui) = ui.as_ref() {
                        ui.hide(path);
                    }
                }
                EventPipelineStep::BurstParticles { emitter, count } => {
                    if let Some(particles) = particles.as_ref() {
                        particles.burst(&emitter, count);
                    }
                }
                EventPipelineStep::TransitionScene { scene } => {
                    if let Some(scene_commands) = scene_commands.as_ref() {
                        scene_commands.submit(SceneCommand::SelectScene {
                            scene: SceneKey::new(scene),
                        });
                    }
                }
                EventPipelineStep::EmitEvent { topic, payload } => {
                    if let Some(script_events) = script_events.as_ref() {
                        script_events.publish(ScriptEvent::new(topic, payload));
                    }
                }
                EventPipelineStep::Script { function } => {
                    if let Some(script_runtime) = script_runtime.as_ref() {
                        run_script(script_runtime.as_ref(), &function, event)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn register_audio_clip_reference(
    asset_catalog: &AssetCatalog,
    audio_scene_service: &AudioSceneService,
    asset_key: &AssetKey,
    mode: AudioPlaybackMode,
) {
    let source_mod = asset_key
        .as_str()
        .split('/')
        .next()
        .unwrap_or_default()
        .to_owned();
    if source_mod.is_empty() {
        return;
    }

    asset_catalog.register_manifest(AssetManifest {
        key: asset_key.clone(),
        source: AssetSourceKind::Mod(source_mod),
        tags: vec![
            "phase3".to_owned(),
            "audio".to_owned(),
            "generated".to_owned(),
        ],
    });
    asset_catalog.request_load(AssetLoadRequest::new(
        asset_key.clone(),
        AssetLoadPriority::Interactive,
    ));
    audio_scene_service.register_clip(AudioClip {
        key: AudioClipKey::new(asset_key.as_str().to_owned()),
        mode,
    });
}

fn set_state_from_string(state: &SceneStateService, key: String, value: String) {
    if value.eq_ignore_ascii_case("true") {
        state.set_bool(key, true);
    } else if value.eq_ignore_ascii_case("false") {
        state.set_bool(key, false);
    } else if let Ok(value) = value.parse::<i64>() {
        state.set_int(key, value);
    } else if let Ok(value) = value.parse::<f64>() {
        state.set_float(key, value);
    } else {
        state.set_string(key, value);
    };
}
