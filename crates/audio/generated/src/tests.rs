use std::collections::BTreeMap;
use std::path::PathBuf;

use amigo_assets::{
    AssetKey, AssetSourceKind, LoadedAsset, PreparedAsset, PreparedAssetKind,
    prepare_asset_from_contents,
};

use super::{
    DEFAULT_AUDIO_SAMPLE_RATE, GeneratedAudioClip, PcSpeakerGenerator, PcSpeakerRealtimeState, Waveform,
    parse_generated_audio_asset,
};

fn prepared_generated_audio(contents: &str) -> PreparedAsset {
    let loaded = LoadedAsset {
        key: AssetKey::new("playground-sidescroller/audio/test"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from("mods/playground-sidescroller/audio/test.yml"),
        byte_len: contents.len() as u64,
    };
    let prepared = prepare_asset_from_contents(&loaded, contents)
        .expect("generated audio metadata should parse");
    assert_eq!(prepared.kind, PreparedAssetKind::GeneratedAudio);
    prepared
}

#[test]
fn reports_total_sequence_duration() {
    let prepared = prepared_generated_audio(
        r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 44100
sequence:
  - wave: square
    frequency: 330
    duration_ms: 40
    volume: 0.35
  - wave: square
    frequency: 520
    duration_ms: 90
    volume: 0.30
envelope:
  attack_ms: 2
  release_ms: 30
"#,
    );

    let GeneratedAudioClip::Pregenerated(clip) =
        parse_generated_audio_asset(&prepared).expect("clip should parse")
    else {
        panic!("expected pregenerated clip");
    };

    assert_eq!(clip.sequence.total_duration_ms(), 130);
}

#[test]
fn generates_expected_sample_count_for_square_wave_clip() {
    let prepared = prepared_generated_audio(
        r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 1000
sequence:
  - wave: square
    frequency: 100
    duration_ms: 10
    volume: 0.5
envelope:
  attack_ms: 0
  release_ms: 0
"#,
    );

    let GeneratedAudioClip::Pregenerated(clip) =
        parse_generated_audio_asset(&prepared).expect("clip should parse")
    else {
        panic!("expected pregenerated clip");
    };
    let pcm = PcSpeakerGenerator::generate_clip(&clip);

    assert_eq!(pcm.sample_rate, 1000);
    assert_eq!(pcm.samples.len(), 10);
    assert!(pcm.samples.iter().all(|sample| sample.abs() <= 0.5));
}

#[test]
fn envelope_shapes_tone_amplitude() {
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
    volume: 1.0
envelope:
  attack_ms: 5
  release_ms: 5
"#,
    );

    let GeneratedAudioClip::Pregenerated(clip) =
        parse_generated_audio_asset(&prepared).expect("clip should parse")
    else {
        panic!("expected pregenerated clip");
    };
    let pcm = PcSpeakerGenerator::generate_clip(&clip);

    assert!(pcm.samples.first().copied().unwrap_or_default().abs() < 0.5);
    assert!(pcm.samples.last().copied().unwrap_or_default().abs() < 0.5);
    assert!(
        pcm.samples
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0_f32, f32::max)
            <= 1.0
    );
}

#[test]
fn parses_realtime_generated_audio_and_maps_distance_deterministically() {
    let prepared = prepared_generated_audio(
        r#"
kind: generated-audio
generator: pc-speaker
mode: realtime
wave: square
volume: 0.20
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

    let GeneratedAudioClip::Realtime(clip) =
        parse_generated_audio_asset(&prepared).expect("clip should parse")
    else {
        panic!("expected realtime clip");
    };
    assert_eq!(clip.sample_rate, DEFAULT_AUDIO_SAMPLE_RATE);
    assert_eq!(clip.wave, Waveform::Square);

    let mut state = PcSpeakerRealtimeState::default();
    let near = PcSpeakerGenerator::generate_realtime_frame(
        &clip,
        Some(&BTreeMap::from([("distance".to_owned(), 0.0)])),
        &mut state,
        256,
    );
    let far = PcSpeakerGenerator::generate_realtime_frame(
        &clip,
        Some(&BTreeMap::from([("distance".to_owned(), 512.0)])),
        &mut state,
        256,
    );

    assert_eq!(near.len(), 256);
    assert_eq!(far.len(), 256);
    assert!(near.iter().any(|sample| sample.abs() > 0.0));
}
