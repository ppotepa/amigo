use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

pub type ScriptParams = BTreeMap<String, ScriptValue>;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptComponentDefinition {
    pub source_mod: String,
    pub entity_name: String,
    pub source_name: String,
    pub script: std::path::PathBuf,
    pub params: ScriptParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptCommand {
    pub namespace: String,
    pub name: String,
    pub arguments: Vec<String>,
}

impl ScriptCommand {
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<Vec<String>>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }

    pub fn ui_set_text(path: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new("ui", "set-text", vec![path.into(), value.into()])
    }

    pub fn ui_set_value(path: impl Into<String>, value: f32) -> Self {
        Self::new("ui", "set-value", vec![path.into(), value.to_string()])
    }

    pub fn ui_set_color(path: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new("ui", "set-color", vec![path.into(), value.into()])
    }

    pub fn ui_show(path: impl Into<String>) -> Self {
        Self::new("ui", "show", vec![path.into()])
    }

    pub fn ui_hide(path: impl Into<String>) -> Self {
        Self::new("ui", "hide", vec![path.into()])
    }

    pub fn ui_enable(path: impl Into<String>) -> Self {
        Self::new("ui", "enable", vec![path.into()])
    }

    pub fn ui_disable(path: impl Into<String>) -> Self {
        Self::new("ui", "disable", vec![path.into()])
    }

    pub fn audio_play(clip_name: impl Into<String>) -> Self {
        Self::new("audio", "play", vec![clip_name.into()])
    }

    pub fn audio_play_asset(asset_key: impl Into<String>) -> Self {
        Self::new("audio", "play-asset", vec![asset_key.into()])
    }

    pub fn audio_cue(cue_name: impl Into<String>) -> Self {
        Self::new("audio", "cue", vec![cue_name.into()])
    }

    pub fn scene_activate_set(set_id: impl Into<String>) -> Self {
        Self::new("scene", "activate-set", vec![set_id.into()])
    }

    pub fn audio_preload(clip_name: impl Into<String>) -> Self {
        Self::new("audio", "preload", vec![clip_name.into()])
    }

    pub fn audio_start_realtime(source: impl Into<String>) -> Self {
        Self::new("audio", "start-realtime", vec![source.into()])
    }

    pub fn audio_stop(source: impl Into<String>) -> Self {
        Self::new("audio", "stop", vec![source.into()])
    }

    pub fn audio_set_param(
        source: impl Into<String>,
        param: impl Into<String>,
        value: f32,
    ) -> Self {
        Self::new(
            "audio",
            "set-param",
            vec![source.into(), param.into(), value.to_string()],
        )
    }

    pub fn audio_set_volume(bus: impl Into<String>, value: f32) -> Self {
        Self::new("audio", "set-volume", vec![bus.into(), value.to_string()])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptEvent {
    pub topic: String,
    pub payload: Vec<String>,
}

impl ScriptEvent {
    pub fn new(topic: impl Into<String>, payload: impl Into<Vec<String>>) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevConsoleCommand {
    pub line: String,
}

impl DevConsoleCommand {
    pub fn new(line: impl Into<String>) -> Self {
        Self { line: line.into() }
    }
}
