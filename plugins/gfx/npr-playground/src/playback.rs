//! Transient, single-model transport. Never enters drawing documents or undo history.
use crate::state::Settings;
use amigo_3d_mesh::MeshAnimationClip;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlaybackSource {
    #[default]
    Turntable,
    Clip {
        index: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlaybackCommand {
    Source { source: PlaybackSource },
    Playing { playing: bool },
    Seek { seconds: f32 },
    Options { looping: bool, speed: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelPlayback {
    pub source: PlaybackSource,
    pub playing: bool,
    pub time_seconds: f32,
    pub duration_seconds: Option<f32>,
    pub looping: bool,
    pub speed: f32,
    #[serde(skip)]
    rotation_origin: Vec3,
}
impl ModelPlayback {
    pub fn for_settings(settings: &Settings) -> Self {
        let object = settings.objects.get(&settings.selected);
        Self {
            source: PlaybackSource::Turntable,
            playing: object.is_some_and(|o| o.rotating) && !settings.paused && settings.speed > 0.0,
            time_seconds: 0.0,
            duration_seconds: None,
            looping: true,
            speed: if settings.speed > 0.0 {
                settings.speed.clamp(0.05, 4.0)
            } else {
                1.0
            },
            rotation_origin: object.map_or(Vec3::ZERO, |o| o.rotation),
        }
    }
    pub fn apply(
        &self,
        command: PlaybackCommand,
        settings: &Settings,
        clips: &[MeshAnimationClip],
    ) -> Result<Self, String> {
        if !settings.objects.contains_key(&settings.selected) {
            return Err("no active model".into());
        }
        let mut next = self.clone();
        match command {
            PlaybackCommand::Source { source } => {
                next.duration_seconds = match source {
                    PlaybackSource::Turntable => None,
                    PlaybackSource::Clip { index } => Some(
                        clips
                            .iter()
                            .find(|clip| clip.index == index)
                            .ok_or("animation clip does not exist")?
                            .duration_seconds,
                    ),
                };
                next.source = source;
                next.time_seconds = 0.0;
                next.playing = false;
                next.rotation_origin = settings.objects[&settings.selected].rotation;
            }
            PlaybackCommand::Playing { playing } => {
                if playing && next.duration_seconds == Some(0.0) {
                    return Err("this clip contains a constant pose, not a timed animation".into());
                }
                if playing
                    && next
                        .duration_seconds
                        .is_some_and(|duration| next.time_seconds >= duration)
                {
                    next.time_seconds = 0.0;
                }
                next.playing = playing;
            }
            PlaybackCommand::Seek { seconds } => {
                if !seconds.is_finite()
                    || seconds < 0.0
                    || seconds > 1_000_000.0
                    || next
                        .duration_seconds
                        .is_some_and(|duration| seconds > duration)
                {
                    return Err("seek is outside the playback range".into());
                }
                next.time_seconds = seconds;
            }
            PlaybackCommand::Options { looping, speed } => {
                if !speed.is_finite() || !(0.05..=4.0).contains(&speed) {
                    return Err("playback speed must be between 0.05 and 4".into());
                }
                next.looping = looping;
                next.speed = speed;
            }
        }
        Ok(next)
    }
    pub fn advance(&mut self, dt: f32) {
        if !self.playing || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let time = self.time_seconds as f64 + dt as f64 * self.speed as f64;
        self.time_seconds = if let Some(duration) = self.duration_seconds {
            if duration <= 0.0 {
                self.playing = false;
                0.0
            } else if self.looping {
                time.rem_euclid(duration as f64) as f32
            } else if time >= duration as f64 {
                self.playing = false;
                duration
            } else {
                time as f32
            }
        } else {
            time.min(1_000_000.0) as f32
        };
    }
    pub fn rotation(&self, speed: Vec3) -> Vec3 {
        (self.rotation_origin + speed * self.time_seconds).map(|v| v.rem_euclid(360.0))
    }
    pub fn rebase_rotation(&mut self, rotation: Vec3, speed: Vec3) {
        self.rotation_origin = rotation - speed * self.time_seconds;
    }
}
