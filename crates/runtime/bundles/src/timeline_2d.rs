use std::sync::Mutex;

use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;
use amigo_runtime_control::{ControlValue, RuntimeControlService};
use amigo_state::SceneStateService;
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct Timeline2dService {
    timelines: Mutex<Vec<Timeline2dDocument>>,
}

impl Timeline2dService {
    pub fn replace_timelines(&self, timelines: Vec<Timeline2dDocument>) {
        *self
            .timelines
            .lock()
            .expect("timeline 2d service mutex should not be poisoned") = timelines;
    }

    pub fn clear(&self) {
        self.replace_timelines(Vec::new());
    }

    pub fn timelines(&self) -> Vec<Timeline2dDocument> {
        self.timelines
            .lock()
            .expect("timeline 2d service mutex should not be poisoned")
            .clone()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timeline2dDocument {
    pub kind: String,
    #[serde(default)]
    pub schema_version: u32,
    pub id: String,
    pub clock: Timeline2dClock,
    #[serde(default)]
    pub beats: Vec<Timeline2dBeat>,
    #[serde(default)]
    pub tracks: Vec<Timeline2dTrack>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timeline2dClock {
    pub state_key: String,
    pub complete_at_s: f64,
    #[serde(default)]
    pub complete_state_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timeline2dBeat {
    pub id: String,
    #[serde(default)]
    pub start_s: f64,
    #[serde(default)]
    pub duration_s: Option<f64>,
    #[serde(default)]
    pub pulse: Option<Timeline2dPulse>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timeline2dTrack {
    pub id: String,
    #[serde(default)]
    pub control_path: Option<String>,
    #[serde(default)]
    pub state_key: Option<String>,
    #[serde(default)]
    pub curve: Vec<Timeline2dKeyframe>,
    #[serde(default)]
    pub pulse: Option<Timeline2dPulseTrack>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timeline2dKeyframe {
    pub t: f64,
    pub value: f64,
    #[serde(default)]
    pub easing: Timeline2dEasing,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timeline2dPulse {
    pub rise_s: f64,
    pub hold_s: f64,
    pub decay_s: f64,
    #[serde(default = "default_pulse_value")]
    pub value: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timeline2dPulseTrack {
    pub start_s: f64,
    pub rise_s: f64,
    pub hold_s: f64,
    pub decay_s: f64,
    #[serde(default = "default_pulse_value")]
    pub value: f64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Timeline2dEasing {
    #[default]
    Linear,
    Smoothstep,
}

pub fn load_timeline_2d_document(path: &std::path::Path) -> AmigoResult<Timeline2dDocument> {
    let raw = std::fs::read_to_string(path)?;
    let document = serde_yaml::from_str::<Timeline2dDocument>(&raw).map_err(|error| {
        AmigoError::Message(format!(
            "failed to parse timeline-2d `{}`: {error}",
            path.display()
        ))
    })?;
    validate_timeline_2d_document(&document).map_err(|error| {
        AmigoError::Message(format!("invalid timeline-2d `{}`: {error}", path.display()))
    })?;
    Ok(document)
}

pub fn tick_timeline_2d_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    if delta_seconds <= 0.0 || !delta_seconds.is_finite() {
        return Ok(());
    }
    let Some(timelines) = runtime.resolve::<Timeline2dService>() else {
        return Ok(());
    };
    let Some(state) = runtime.resolve::<SceneStateService>() else {
        return Ok(());
    };
    let Some(control) = runtime.resolve::<RuntimeControlService>() else {
        return Ok(());
    };

    for timeline in timelines.timelines() {
        let complete_key = timeline
            .clock
            .complete_state_key
            .as_deref()
            .unwrap_or("intro.complete");
        if state.get_bool(complete_key).unwrap_or(false) {
            continue;
        }
        let now = state
            .add_float(timeline.clock.state_key.clone(), f64::from(delta_seconds))
            .min(timeline.clock.complete_at_s);
        for track in &timeline.tracks {
            let Some(value) = track_value(track, now) else {
                continue;
            };
            if let Some(path) = track.control_path.as_deref() {
                control
                    .set(path, ControlValue::F64(value))
                    .map_err(|error| {
                        AmigoError::Message(format!(
                            "timeline `{}` track `{}` failed to set `{path}`: {error}",
                            timeline.id, track.id
                        ))
                    })?;
            }
            if let Some(key) = track.state_key.as_deref() {
                state.set_float(key, value);
            }
        }
        if now >= timeline.clock.complete_at_s {
            state.set_bool(complete_key, true);
        }
    }
    Ok(())
}

fn validate_timeline_2d_document(document: &Timeline2dDocument) -> Result<(), String> {
    if document.kind != "timeline-2d" {
        return Err("kind must be timeline-2d".to_owned());
    }
    if document.schema_version != 1 {
        return Err("schema_version must be 1".to_owned());
    }
    if document.id.is_empty() {
        return Err("id must not be empty".to_owned());
    }
    if document.clock.state_key.is_empty() {
        return Err("clock.state_key must not be empty".to_owned());
    }
    if !document.clock.complete_at_s.is_finite() || document.clock.complete_at_s <= 0.0 {
        return Err("clock.complete_at_s must be finite and > 0".to_owned());
    }
    for track in &document.tracks {
        if track.id.is_empty() {
            return Err("track id must not be empty".to_owned());
        }
        if track.control_path.is_none() && track.state_key.is_none() {
            return Err(format!(
                "track `{}` must declare control_path or state_key",
                track.id
            ));
        }
        if track.curve.is_empty() && track.pulse.is_none() {
            return Err(format!("track `{}` must declare curve or pulse", track.id));
        }
    }
    Ok(())
}

fn track_value(track: &Timeline2dTrack, time: f64) -> Option<f64> {
    if let Some(pulse) = &track.pulse {
        return Some(pulse_value(
            time,
            pulse.start_s,
            pulse.rise_s,
            pulse.hold_s,
            pulse.decay_s,
            pulse.value,
        ));
    }
    curve_value(&track.curve, time)
}

fn curve_value(keyframes: &[Timeline2dKeyframe], time: f64) -> Option<f64> {
    let first = keyframes.first()?;
    if time <= first.t {
        return Some(first.value);
    }
    for pair in keyframes.windows(2) {
        let start = &pair[0];
        let end = &pair[1];
        if time <= end.t {
            let duration = (end.t - start.t).max(f64::EPSILON);
            let mut t = ((time - start.t) / duration).clamp(0.0, 1.0);
            if end.easing == Timeline2dEasing::Smoothstep {
                t = smooth01(t);
            }
            return Some(start.value + (end.value - start.value) * t);
        }
    }
    keyframes.last().map(|keyframe| keyframe.value)
}

fn pulse_value(time: f64, start: f64, rise: f64, hold: f64, decay: f64, value: f64) -> f64 {
    if time < start {
        return 0.0;
    }
    let t = time - start;
    if t < rise {
        return smooth01(t / rise.max(f64::EPSILON)) * value;
    }
    if t < rise + hold {
        return value;
    }
    if t < rise + hold + decay {
        return (1.0 - smooth01((t - rise - hold) / decay.max(f64::EPSILON))) * value;
    }
    0.0
}

fn smooth01(value: f64) -> f64 {
    let t = value.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn default_pulse_value() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_value_uses_smoothstep_keyframe_easing() {
        let value = curve_value(
            &[
                Timeline2dKeyframe {
                    t: 0.0,
                    value: 0.0,
                    easing: Timeline2dEasing::Linear,
                },
                Timeline2dKeyframe {
                    t: 2.0,
                    value: 10.0,
                    easing: Timeline2dEasing::Smoothstep,
                },
            ],
            1.0,
        );
        assert_eq!(value, Some(5.0));
    }

    #[test]
    fn pulse_value_uses_rise_hold_decay() {
        assert_eq!(pulse_value(0.0, 1.0, 0.5, 0.5, 1.0, 2.0), 0.0);
        assert_eq!(pulse_value(1.5, 1.0, 0.5, 0.5, 1.0, 2.0), 2.0);
        assert_eq!(pulse_value(3.1, 1.0, 0.5, 0.5, 1.0, 2.0), 0.0);
    }
}
