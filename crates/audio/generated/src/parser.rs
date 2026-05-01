use std::collections::BTreeMap;

use amigo_assets::{PreparedAsset, PreparedAssetKind};

use crate::types::{
    GeneratedAudioClip, GeneratedAudioMode, GeneratedAudioParamMapping, GeneratedAudioParamSpec, Envelope,
    DEFAULT_AUDIO_SAMPLE_RATE, PregeneratedGeneratedAudioClip, RealtimeGeneratedAudioClip, Tone, ToneSequence,
    Waveform,
};

pub fn parse_generated_audio_asset(prepared: &PreparedAsset) -> Result<GeneratedAudioClip, String> {
    if prepared.kind != PreparedAssetKind::GeneratedAudio {
        return Err(format!(
            "asset `{}` is not generated-audio",
            prepared.key.as_str()
        ));
    }

    let generator = metadata_string(prepared, "generator")?;
    let mode = match metadata_string(prepared, "mode")?.as_str() {
        "pregenerated" => GeneratedAudioMode::Pregenerated,
        "realtime" => GeneratedAudioMode::Realtime,
        other => return Err(format!("unsupported generated-audio mode `{other}`")),
    };

    match mode {
        GeneratedAudioMode::Pregenerated => Ok(GeneratedAudioClip::Pregenerated(
            PregeneratedGeneratedAudioClip {
                generator,
                sample_rate: metadata_u32(prepared, "sample_rate")?.unwrap_or(DEFAULT_AUDIO_SAMPLE_RATE),
                sequence: ToneSequence {
                    tones: parse_tone_sequence(prepared)?,
                },
                envelope: Envelope {
                    attack_ms: metadata_u32(prepared, "envelope.attack_ms")?.unwrap_or(0),
                    release_ms: metadata_u32(prepared, "envelope.release_ms")?.unwrap_or(0),
                },
            },
        )),
        GeneratedAudioMode::Realtime => Ok(GeneratedAudioClip::Realtime(RealtimeGeneratedAudioClip {
            generator,
            sample_rate: metadata_u32(prepared, "sample_rate")?.unwrap_or(DEFAULT_AUDIO_SAMPLE_RATE),
            wave: Waveform::parse(&metadata_string(prepared, "wave")?)?,
            volume: metadata_f32(prepared, "volume")?
                .unwrap_or(0.25)
                .clamp(0.0, 1.0),
            params: parse_param_specs(prepared),
            interval_ms: parse_mapping(prepared, "mapping.interval_ms")?,
            frequency: parse_mapping(prepared, "mapping.frequency")?,
        })),
    }
}

fn parse_tone_sequence(prepared: &PreparedAsset) -> Result<Vec<Tone>, String> {
    let mut tones = Vec::new();

    for index in 0.. {
        let wave_key = format!("sequence.{index}.wave");
        let Some(wave_value) = prepared.metadata.get(&wave_key) else {
            break;
        };
        tones.push(Tone {
            wave: Waveform::parse(wave_value)?,
            frequency: metadata_f32(prepared, &format!("sequence.{index}.frequency"))?
                .ok_or_else(|| format!("missing `sequence.{index}.frequency`"))?,
            duration_ms: metadata_u32(prepared, &format!("sequence.{index}.duration_ms"))?
                .ok_or_else(|| format!("missing `sequence.{index}.duration_ms`"))?,
            volume: metadata_f32(prepared, &format!("sequence.{index}.volume"))?
                .unwrap_or(0.25)
                .clamp(0.0, 1.0),
        });
    }

    if tones.is_empty() {
        return Err(format!(
            "generated-audio asset `{}` is missing `sequence` entries",
            prepared.key.as_str()
        ));
    }

    Ok(tones)
}

fn parse_param_specs(prepared: &PreparedAsset) -> BTreeMap<String, GeneratedAudioParamSpec> {
    let mut params = BTreeMap::new();

    for key in prepared.metadata.keys() {
        let Some(rest) = key.strip_prefix("params.") else {
            continue;
        };
        let Some((param_name, field_name)) = rest.split_once('.') else {
            continue;
        };
        let entry = params
            .entry(param_name.to_owned())
            .or_insert(GeneratedAudioParamSpec {
                default: 0.0,
                min: 0.0,
                max: 1.0,
            });

        let value = metadata_f32(prepared, key).ok().flatten().unwrap_or_default();
        match field_name {
            "default" => entry.default = value,
            "min" => entry.min = value,
            "max" => entry.max = value,
            _ => {}
        }
    }

    params
}

fn parse_mapping(
    prepared: &PreparedAsset,
    prefix: &str,
) -> Result<GeneratedAudioParamMapping, String> {
    Ok(GeneratedAudioParamMapping {
        from_param: metadata_string(prepared, &format!("{prefix}.from_param"))?,
        near_value: metadata_f32(prepared, &format!("{prefix}.near_value"))?
            .ok_or_else(|| format!("missing `{prefix}.near_value`"))?,
        far_value: metadata_f32(prepared, &format!("{prefix}.far_value"))?
            .ok_or_else(|| format!("missing `{prefix}.far_value`"))?,
    })
}

pub(crate) fn mapped_value(
    mapping: &GeneratedAudioParamMapping,
    specs: &BTreeMap<String, GeneratedAudioParamSpec>,
    params: Option<&BTreeMap<String, f32>>,
) -> f32 {
    let spec = specs.get(&mapping.from_param);
    let (default, min, max) = spec
        .map(|spec| (spec.default, spec.min, spec.max))
        .unwrap_or((0.0, 0.0, 1.0));
    let raw_value = params
        .and_then(|params| params.get(&mapping.from_param).copied())
        .unwrap_or(default);
    let clamped = raw_value.clamp(min, max);
    let normalized = if (max - min).abs() <= f32::EPSILON {
        0.0
    } else {
        (clamped - min) / (max - min)
    };
    mapping.near_value + (mapping.far_value - mapping.near_value) * normalized
}

fn metadata_string(prepared: &PreparedAsset, key: &str) -> Result<String, String> {
    prepared
        .metadata
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing `{key}` in `{}`", prepared.key.as_str()))
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Result<Option<u32>, String> {
    match prepared.metadata.get(key) {
        Some(value) => value.parse::<u32>().map(Some).map_err(|error| {
            format!(
                "invalid u32 `{key}` in `{}`: {error}",
                prepared.key.as_str()
            )
        }),
        None => Ok(None),
    }
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Result<Option<f32>, String> {
    match prepared.metadata.get(key) {
        Some(value) => value.parse::<f32>().map(Some).map_err(|error| {
            format!(
                "invalid f32 `{key}` in `{}`: {error}",
                prepared.key.as_str()
            )
        }),
        None => Ok(None),
    }
}
