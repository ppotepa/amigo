use amigo_2d_particles::Particle2dSceneService;
use amigo_assets::AssetKey;
use amigo_audio_api::{AudioCommand, AudioCommandQueue, AudioPlaybackMode};
use amigo_core::AmigoResult;
use amigo_event_pipeline::{EventPipelineService, EventPipelineStep};
use amigo_runtime::Runtime;
use amigo_scene::{SceneCommand, SceneCommandQueue, SceneKey};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};
use amigo_state::SceneStateService;
use amigo_ui::UiStateService;

use crate::LaunchSelection;
use crate::runtime_context::RuntimeContext;

pub(crate) fn run_event_pipelines_for_event(
    runtime: &Runtime,
    event: &ScriptEvent,
) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let pipelines = ctx.required::<EventPipelineService>()?;
    let state = ctx.required::<SceneStateService>()?;
    let ui = ctx.optional::<UiStateService>();
    let particles = ctx.optional::<Particle2dSceneService>();
    let audio_commands = ctx.optional::<AudioCommandQueue>();
    let scene_commands = ctx.optional::<SceneCommandQueue>();
    let script_events = ctx.optional::<ScriptEventQueue>();
    let launch_selection = ctx.optional::<LaunchSelection>();
    let asset_catalog = ctx.optional::<amigo_assets::AssetCatalog>();
    let audio_scene = ctx.optional::<amigo_audio_api::AudioSceneService>();

    for pipeline in pipelines.pipelines_for_topic(&event.topic) {
        for step in pipeline.steps {
            match step {
                EventPipelineStep::PlayAudio { clip } => {
                    if let Some(audio_commands) = audio_commands.as_ref() {
                        let asset_key = launch_selection
                            .as_ref()
                            .map(|selection| {
                                crate::app_helpers::resolve_mod_audio_asset_key(selection, &clip)
                            })
                            .unwrap_or_else(|| AssetKey::new(clip.clone()));
                        if let (Some(asset_catalog), Some(audio_scene)) =
                            (asset_catalog.as_ref(), audio_scene.as_ref())
                        {
                            crate::app_helpers::register_audio_clip_reference(
                                asset_catalog.as_ref(),
                                audio_scene.as_ref(),
                                &asset_key,
                                AudioPlaybackMode::OneShot,
                            );
                        }
                        audio_commands.push(AudioCommand::PlayOnce {
                            clip: amigo_audio_api::AudioClipKey::new(asset_key.as_str()),
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
            }
        }
    }

    Ok(())
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
