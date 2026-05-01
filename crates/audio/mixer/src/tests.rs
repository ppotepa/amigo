mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use amigo_assets::{AssetKey, AssetSourceKind, LoadedAsset, prepare_asset_from_contents};
    use amigo_audio_api::AudioClipKey;

    use super::AudioMixerService;

    fn prepared_generated_audio(contents: &str) -> amigo_assets::PreparedAsset {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/audio/test"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/audio/test.yml"),
            byte_len: contents.len() as u64,
        };
        prepare_asset_from_contents(&loaded, contents)
            .expect("generated audio metadata should parse")
    }

    #[test]
    fn queues_and_ticks_generated_one_shot_audio() {
        let prepared = prepared_generated_audio(
            r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 1000
sequence:
  - wave: square
    frequency: 100
    duration_ms: 20
    volume: 0.5
envelope:
  attack_ms: 0
  release_ms: 0
"#,
        );
        let service = AudioMixerService::default();

        service
            .queue_generated_one_shot("jump", &prepared)
            .expect("one-shot should queue");
        let frame = service
            .tick_generated_audio(
                &BTreeMap::new(),
                &BTreeMap::new(),
                &BTreeMap::new(),
                1.0,
                16,
            )
            .expect("mix frame should be produced");

        assert_eq!(frame.sample_rate, 44_100);
        assert_eq!(frame.samples.len(), 16);
        assert!(frame.sources.iter().any(|source| source == "jump"));
    }

    #[test]
    fn ticks_realtime_generated_audio_source() {
        let prepared = prepared_generated_audio(
            r#"
kind: generated-audio
generator: pc-speaker
mode: realtime
wave: square
volume: 0.2
params:
  distance:
    default: 256.0
    min: 0.0
    max: 512.0
mapping:
  interval_ms:
    from_param: distance
    near_value: 80
    far_value: 900
  frequency:
    from_param: distance
    near_value: 1200
    far_value: 320
"#,
        );
        let service = AudioMixerService::default();
        let prepared_assets =
            BTreeMap::from([(prepared.key.as_str().to_owned(), prepared.clone())]);
        let playing_sources = BTreeMap::from([(
            "proximity-beep".to_owned(),
            AudioClipKey::new("playground-sidescroller/audio/test"),
        )]);
        let source_params = BTreeMap::from([(
            "proximity-beep".to_owned(),
            BTreeMap::from([("distance".to_owned(), 32.0)]),
        )]);

        let frame = service
            .tick_generated_audio(&prepared_assets, &playing_sources, &source_params, 1.0, 128)
            .expect("realtime frame should be produced");

        assert_eq!(frame.samples.len(), 128);
        assert!(
            frame
                .sources
                .iter()
                .any(|source| source == "proximity-beep")
        );
        assert!(
            service
                .active_realtime_sources()
                .iter()
                .any(|source| source == "proximity-beep")
        );
    }
}
