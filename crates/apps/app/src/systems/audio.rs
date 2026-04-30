use std::collections::BTreeMap;

use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_audio_runtime(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let asset_catalog = ctx.required::<AssetCatalog>()?;
    let audio_state_service = ctx.required::<AudioStateService>()?;
    let audio_mixer_service = ctx.required::<AudioMixerService>()?;
    let audio_output_service = ctx.required::<AudioOutputBackendService>()?;
    let dev_console_state = ctx.required::<DevConsoleState>()?;

    let prepared_assets = asset_catalog
        .prepared_assets()
        .into_iter()
        .map(|asset| (asset.key.as_str().to_owned(), asset))
        .collect::<BTreeMap<_, _>>();
    let playing_sources = audio_state_service.playing_sources();
    let source_params = audio_state_service.source_params();
    let frame_sample_count = ((44_100.0 * delta_seconds.max(0.0)).round() as usize).max(1);
    let mut deferred_commands = Vec::new();

    for command in audio_state_service.drain_runtime_commands() {
        if let AudioCommand::PlayOnce { clip } = command {
            if let Some(prepared_asset) = prepared_assets.get(clip.as_str()) {
                audio_state_service.clear_deferred_one_shot_logged(clip.as_str());
                if let Err(error) = audio_mixer_service
                    .queue_generated_one_shot(clip.as_str().to_owned(), prepared_asset)
                {
                    dev_console_state.write_line(format!(
                        "audio runtime queue failed for `{}`: {error}",
                        clip.as_str()
                    ));
                }
            } else {
                if audio_state_service.mark_deferred_one_shot_logged(clip.as_str()) {
                    dev_console_state.write_line(format!(
                        "audio deferred one-shot `{}` until asset is prepared",
                        clip.as_str()
                    ));
                }
                deferred_commands.push(AudioCommand::PlayOnce { clip });
            }
        }
    }

    for command in deferred_commands {
        audio_state_service.queue_runtime_command(command);
    }

    if let Some(frame) = audio_mixer_service.tick_generated_audio(
        &prepared_assets,
        &playing_sources,
        &source_params,
        audio_state_service.master_volume(),
        frame_sample_count,
    ) {
        if audio_state_service.mark_first_mix_frame_logged() {
            dev_console_state.write_line(format!(
                "audio mixed first frame: samples={} sources={}",
                frame.samples.len(),
                frame.sources.join(", ")
            ));
        }
        audio_output_service.enqueue_mix_frame(&frame);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use amigo_assets::{
        AssetCatalog, AssetKey, AssetSourceKind, AssetsPlugin, LoadedAsset,
        prepare_asset_from_contents,
    };
    use amigo_audio_api::{AudioApiPlugin, AudioClipKey, AudioCommand, AudioStateService};
    use amigo_audio_mixer::{AudioMixerPlugin, AudioMixerService};
    use amigo_audio_output::{AudioOutputBackendService, AudioOutputPlugin};
    use amigo_core::AmigoResult;
    use amigo_runtime::{RuntimeBuilder, RuntimePlugin, ServiceRegistry};
    use amigo_scripting_api::DevConsoleState;

    use super::tick_audio_runtime;

    struct TestConsolePlugin;

    impl RuntimePlugin for TestConsolePlugin {
        fn name(&self) -> &'static str {
            "test-dev-console"
        }

        fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
            registry.register(DevConsoleState::default())
        }
    }

    fn prepared_generated_audio(asset_key: &str) -> amigo_assets::PreparedAsset {
        let loaded = LoadedAsset {
            key: AssetKey::new(asset_key),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/audio/jump.yml"),
            byte_len: 0,
        };
        prepare_asset_from_contents(
            &loaded,
            r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 1000
sequence:
  - wave: square
    frequency: 440
    duration_ms: 20
    volume: 0.5
envelope:
  attack_ms: 0
  release_ms: 0
"#,
        )
        .expect("generated audio metadata should parse")
    }

    #[test]
    fn defers_play_once_until_asset_is_prepared() {
        let runtime = RuntimeBuilder::default()
            .with_plugin(AssetsPlugin)
            .expect("assets plugin should register")
            .with_plugin(AudioApiPlugin)
            .expect("audio api plugin should register")
            .with_plugin(AudioMixerPlugin)
            .expect("audio mixer plugin should register")
            .with_plugin(AudioOutputPlugin)
            .expect("audio output plugin should register")
            .with_plugin(TestConsolePlugin)
            .expect("test console plugin should register")
            .build();

        let audio_state = runtime
            .resolve::<AudioStateService>()
            .expect("audio state should exist");
        let asset_catalog = runtime
            .resolve::<AssetCatalog>()
            .expect("asset catalog should exist");
        let mixer = runtime
            .resolve::<AudioMixerService>()
            .expect("audio mixer should exist");
        let output = runtime
            .resolve::<AudioOutputBackendService>()
            .expect("audio output should exist");

        audio_state.queue_runtime_command(AudioCommand::PlayOnce {
            clip: AudioClipKey::new("playground-sidescroller/audio/jump"),
        });

        tick_audio_runtime(&runtime, 1.0 / 60.0).expect("audio tick should succeed");
        assert_eq!(audio_state.pending_runtime_commands().len(), 1);
        assert!(mixer.frames().is_empty());
        assert_eq!(output.snapshot().buffered_samples, 0);

        asset_catalog.mark_prepared(prepared_generated_audio(
            "playground-sidescroller/audio/jump",
        ));

        tick_audio_runtime(&runtime, 1.0 / 60.0).expect("audio tick should succeed after prepare");

        assert!(audio_state.pending_runtime_commands().is_empty());
        assert!(!mixer.frames().is_empty());
        assert!(output.snapshot().buffered_samples > 0);
    }

    #[test]
    fn logs_deferred_one_shot_only_once_until_asset_is_prepared() {
        let runtime = RuntimeBuilder::default()
            .with_plugin(AssetsPlugin)
            .expect("assets plugin should register")
            .with_plugin(AudioApiPlugin)
            .expect("audio api plugin should register")
            .with_plugin(AudioMixerPlugin)
            .expect("audio mixer plugin should register")
            .with_plugin(AudioOutputPlugin)
            .expect("audio output plugin should register")
            .with_plugin(TestConsolePlugin)
            .expect("test console plugin should register")
            .build();

        let audio_state = runtime
            .resolve::<AudioStateService>()
            .expect("audio state should exist");
        let console = runtime
            .resolve::<DevConsoleState>()
            .expect("dev console should exist");

        audio_state.queue_runtime_command(AudioCommand::PlayOnce {
            clip: AudioClipKey::new("playground-sidescroller/audio/jump"),
        });

        tick_audio_runtime(&runtime, 1.0 / 60.0).expect("first audio tick should succeed");
        tick_audio_runtime(&runtime, 1.0 / 60.0).expect("second audio tick should succeed");

        let deferred_logs = console
            .output_lines()
            .into_iter()
            .filter(|line| line.contains("audio deferred one-shot"))
            .collect::<Vec<_>>();
        assert_eq!(deferred_logs.len(), 1);
    }

    #[test]
    fn logs_first_mix_frame_only_once() {
        let runtime = RuntimeBuilder::default()
            .with_plugin(AssetsPlugin)
            .expect("assets plugin should register")
            .with_plugin(AudioApiPlugin)
            .expect("audio api plugin should register")
            .with_plugin(AudioMixerPlugin)
            .expect("audio mixer plugin should register")
            .with_plugin(AudioOutputPlugin)
            .expect("audio output plugin should register")
            .with_plugin(TestConsolePlugin)
            .expect("test console plugin should register")
            .build();

        let audio_state = runtime
            .resolve::<AudioStateService>()
            .expect("audio state should exist");
        let asset_catalog = runtime
            .resolve::<AssetCatalog>()
            .expect("asset catalog should exist");
        let console = runtime
            .resolve::<DevConsoleState>()
            .expect("dev console should exist");

        asset_catalog.mark_prepared(prepared_generated_audio(
            "playground-sidescroller/audio/jump",
        ));
        audio_state.queue_runtime_command(AudioCommand::PlayOnce {
            clip: AudioClipKey::new("playground-sidescroller/audio/jump"),
        });

        tick_audio_runtime(&runtime, 1.0 / 60.0).expect("first mix tick should succeed");
        tick_audio_runtime(&runtime, 1.0 / 60.0).expect("second mix tick should succeed");

        let mix_logs = console
            .output_lines()
            .into_iter()
            .filter(|line| line.contains("audio mixed first frame"))
            .collect::<Vec<_>>();
        assert_eq!(mix_logs.len(), 1);
    }
}
