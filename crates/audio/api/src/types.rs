#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioClipKey(String);

impl AudioClipKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioPlaybackMode {
    OneShot,
    Looping,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioSourceId(String);

impl AudioSourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioClip {
    pub key: AudioClipKey,
    pub mode: AudioPlaybackMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioCue {
    pub name: String,
    pub clip: AudioClipKey,
    pub min_interval_seconds: Option<f32>,
}

impl AudioCue {
    pub fn new(
        name: impl Into<String>,
        clip: AudioClipKey,
        min_interval_seconds: Option<f32>,
    ) -> Self {
        let min_interval_seconds = min_interval_seconds
            .filter(|value| value.is_finite())
            .map(|value| value.max(0.0));
        Self {
            name: name.into(),
            clip,
            min_interval_seconds,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBus {
    pub id: String,
}

impl AudioBus {
    pub fn new(value: impl Into<String>) -> Self {
        Self { id: value.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioCommand {
    PlayOnce {
        clip: AudioClipKey,
    },
    StartSource {
        source: AudioSourceId,
        clip: AudioClipKey,
    },
    StopSource {
        source: AudioSourceId,
    },
    SetParam {
        source: AudioSourceId,
        param: String,
        value: f32,
    },
    SetVolume {
        bus: String,
        value: f32,
    },
    SetMasterVolume {
        value: f32,
    },
}

