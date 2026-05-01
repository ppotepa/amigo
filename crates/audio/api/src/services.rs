use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use std::time::Instant;

use crate::types::{
    AudioClip, AudioClipKey, AudioCommand, AudioCue, AudioSourceId,
};

#[derive(Debug, Default)]
pub struct AudioCommandQueue {
    commands: Mutex<Vec<AudioCommand>>,
}

impl AudioCommandQueue {
    pub fn push(&self, command: AudioCommand) {
        self.commands
            .lock()
            .expect("audio command queue mutex should not be poisoned")
            .push(command);
    }

    pub fn drain(&self) -> Vec<AudioCommand> {
        let mut commands = self
            .commands
            .lock()
            .expect("audio command queue mutex should not be poisoned");
        commands.drain(..).collect()
    }

    pub fn snapshot(&self) -> Vec<AudioCommand> {
        self.commands
            .lock()
            .expect("audio command queue mutex should not be poisoned")
            .clone()
    }
}

#[derive(Debug, Default)]
pub struct AudioSceneService {
    registered_clips: Mutex<Vec<AudioClip>>,
    cues: Mutex<BTreeMap<String, AudioCue>>,
    cue_last_played: Mutex<BTreeMap<String, Instant>>,
}

impl AudioSceneService {
    pub fn register_clip(&self, clip: AudioClip) {
        let mut clips = self
            .registered_clips
            .lock()
            .expect("audio scene service mutex should not be poisoned");
        if clips.iter().any(|existing| existing.key == clip.key) {
            return;
        }
        clips.push(clip);
    }

    pub fn clear(&self) {
        self.registered_clips
            .lock()
            .expect("audio scene service mutex should not be poisoned")
            .clear();
        self.cues
            .lock()
            .expect("audio scene service cue mutex should not be poisoned")
            .clear();
        self.cue_last_played
            .lock()
            .expect("audio scene service cue timer mutex should not be poisoned")
            .clear();
    }

    pub fn clips(&self) -> Vec<AudioClip> {
        self.registered_clips
            .lock()
            .expect("audio scene service mutex should not be poisoned")
            .clone()
    }

    pub fn register_cue(&self, cue: AudioCue) -> bool {
        if cue.name.trim().is_empty() || cue.clip.as_str().trim().is_empty() {
            return false;
        }
        self.cues
            .lock()
            .expect("audio scene service cue mutex should not be poisoned")
            .insert(cue.name.clone(), cue);
        true
    }

    pub fn cue(&self, name: &str) -> Option<AudioCue> {
        self.cues
            .lock()
            .expect("audio scene service cue mutex should not be poisoned")
            .get(name)
            .cloned()
    }

    pub fn cues(&self) -> Vec<AudioCue> {
        self.cues
            .lock()
            .expect("audio scene service cue mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn mark_cue_played_if_ready(&self, cue: &AudioCue) -> bool {
        let Some(min_interval_seconds) = cue.min_interval_seconds else {
            return true;
        };
        let now = Instant::now();
        let mut played = self
            .cue_last_played
            .lock()
            .expect("audio scene service cue timer mutex should not be poisoned");
        if played
            .get(&cue.name)
            .is_some_and(|last| now.duration_since(*last).as_secs_f32() < min_interval_seconds)
        {
            return false;
        }
        played.insert(cue.name.clone(), now);
        true
    }
}

#[derive(Debug)]
pub struct AudioStateService {
    playing_sources: Mutex<BTreeMap<String, AudioClipKey>>,
    source_params: Mutex<BTreeMap<String, BTreeMap<String, f32>>>,
    bus_volumes: Mutex<BTreeMap<String, f32>>,
    master_volume: Mutex<f32>,
    processed_commands: Mutex<Vec<AudioCommand>>,
    pending_runtime_commands: Mutex<Vec<AudioCommand>>,
    deferred_one_shots: Mutex<BTreeSet<String>>,
    first_mix_frame_logged: Mutex<bool>,
}

impl Default for AudioStateService {
    fn default() -> Self {
        Self {
            playing_sources: Mutex::default(),
            source_params: Mutex::default(),
            bus_volumes: Mutex::default(),
            master_volume: Mutex::new(1.0),
            processed_commands: Mutex::default(),
            pending_runtime_commands: Mutex::default(),
            deferred_one_shots: Mutex::default(),
            first_mix_frame_logged: Mutex::default(),
        }
    }
}

impl AudioStateService {
    pub fn start_source(&self, source: AudioSourceId, clip: AudioClipKey) {
        self.playing_sources
            .lock()
            .expect("audio state service source mutex should not be poisoned")
            .insert(source.as_str().to_owned(), clip);
    }

    pub fn stop_source(&self, source: &str) -> bool {
        self.playing_sources
            .lock()
            .expect("audio state service source mutex should not be poisoned")
            .remove(source)
            .is_some()
    }

    pub fn set_param(&self, source: &str, param: impl Into<String>, value: f32) -> bool {
        let param = param.into();
        let mut params = self
            .source_params
            .lock()
            .expect("audio state service param mutex should not be poisoned");
        let entry = params.entry(source.to_owned()).or_default();
        if entry.get(&param).copied() == Some(value) {
            return false;
        }
        entry.insert(param, value);
        true
    }

    pub fn set_volume(&self, bus: &str, value: f32) -> bool {
        let value = value.clamp(0.0, 1.0);
        let mut volumes = self
            .bus_volumes
            .lock()
            .expect("audio state service bus mutex should not be poisoned");
        if volumes.get(bus).copied() == Some(value) {
            return false;
        }
        volumes.insert(bus.to_owned(), value);
        true
    }

    pub fn set_master_volume(&self, value: f32) -> bool {
        let value = value.clamp(0.0, 1.0);
        let mut master = self
            .master_volume
            .lock()
            .expect("audio state service master mutex should not be poisoned");
        if (*master - value).abs() <= f32::EPSILON {
            return false;
        }
        *master = value;
        true
    }

    pub fn master_volume(&self) -> f32 {
        *self
            .master_volume
            .lock()
            .expect("audio state service master mutex should not be poisoned")
    }

    pub fn bus_volumes(&self) -> BTreeMap<String, f32> {
        self.bus_volumes
            .lock()
            .expect("audio state service bus mutex should not be poisoned")
            .clone()
    }

    pub fn source_params(&self) -> BTreeMap<String, BTreeMap<String, f32>> {
        self.source_params
            .lock()
            .expect("audio state service param mutex should not be poisoned")
            .clone()
    }

    pub fn record_processed_command(&self, command: AudioCommand) {
        self.processed_commands
            .lock()
            .expect("audio state service command mutex should not be poisoned")
            .push(command);
    }

    pub fn queue_runtime_command(&self, command: AudioCommand) {
        self.pending_runtime_commands
            .lock()
            .expect("audio state service runtime mutex should not be poisoned")
            .push(command);
    }

    pub fn drain_runtime_commands(&self) -> Vec<AudioCommand> {
        let mut commands = self
            .pending_runtime_commands
            .lock()
            .expect("audio state service runtime mutex should not be poisoned");
        commands.drain(..).collect()
    }

    pub fn pending_runtime_commands(&self) -> Vec<AudioCommand> {
        self.pending_runtime_commands
            .lock()
            .expect("audio state service runtime mutex should not be poisoned")
            .clone()
    }

    pub fn processed_commands(&self) -> Vec<AudioCommand> {
        self.processed_commands
            .lock()
            .expect("audio state service command mutex should not be poisoned")
            .clone()
    }

    pub fn mark_deferred_one_shot_logged(&self, clip: &str) -> bool {
        self.deferred_one_shots
            .lock()
            .expect("audio state service deferred one-shot mutex should not be poisoned")
            .insert(clip.to_owned())
    }

    pub fn clear_deferred_one_shot_logged(&self, clip: &str) {
        self.deferred_one_shots
            .lock()
            .expect("audio state service deferred one-shot mutex should not be poisoned")
            .remove(clip);
    }

    pub fn mark_first_mix_frame_logged(&self) -> bool {
        let mut logged = self
            .first_mix_frame_logged
            .lock()
            .expect("audio state service first mix mutex should not be poisoned");
        if *logged {
            return false;
        }
        *logged = true;
        true
    }

    pub fn clear(&self) {
        self.playing_sources
            .lock()
            .expect("audio state service source mutex should not be poisoned")
            .clear();
        self.source_params
            .lock()
            .expect("audio state service param mutex should not be poisoned")
            .clear();
        self.bus_volumes
            .lock()
            .expect("audio state service bus mutex should not be poisoned")
            .clear();
        self.processed_commands
            .lock()
            .expect("audio state service command mutex should not be poisoned")
            .clear();
        self.pending_runtime_commands
            .lock()
            .expect("audio state service runtime mutex should not be poisoned")
            .clear();
        self.deferred_one_shots
            .lock()
            .expect("audio state service deferred one-shot mutex should not be poisoned")
            .clear();
        *self
            .first_mix_frame_logged
            .lock()
            .expect("audio state service first mix mutex should not be poisoned") = false;
        let _ = self.set_master_volume(1.0);
    }

    pub fn playing_sources(&self) -> BTreeMap<String, AudioClipKey> {
        self.playing_sources
            .lock()
            .expect("audio state service source mutex should not be poisoned")
            .clone()
    }
}

