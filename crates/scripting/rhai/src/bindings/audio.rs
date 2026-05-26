use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_audio_cue, queue_audio_play, queue_audio_play_asset, queue_audio_preload,
    queue_audio_set_param, queue_audio_set_volume, queue_audio_start_realtime, queue_audio_stop,
};

#[derive(Clone)]
pub struct AudioApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl AudioApi {
    pub fn play(&mut self, clip_name: &str) -> bool {
        if clip_name.is_empty() {
            return false;
        }
        queue_audio_play(self.command_queue.as_ref(), clip_name)
    }

    pub fn play_asset(&mut self, asset_key: &str) -> bool {
        if asset_key.is_empty() {
            return false;
        }
        queue_audio_play_asset(self.command_queue.as_ref(), asset_key)
    }

    pub fn cue(&mut self, cue_name: &str) -> bool {
        if cue_name.is_empty() {
            return false;
        }
        queue_audio_cue(self.command_queue.as_ref(), cue_name)
    }

    pub fn preload(&mut self, clip_name: &str) -> bool {
        if clip_name.is_empty() {
            return false;
        }
        queue_audio_preload(self.command_queue.as_ref(), clip_name)
    }

    pub fn start_realtime(&mut self, source: &str) -> bool {
        if source.is_empty() {
            return false;
        }
        queue_audio_start_realtime(self.command_queue.as_ref(), source)
    }

    pub fn stop(&mut self, source: &str) -> bool {
        if source.is_empty() {
            return false;
        }
        queue_audio_stop(self.command_queue.as_ref(), source)
    }

    pub fn set_param(&mut self, source: &str, param: &str, value: rhai::FLOAT) -> bool {
        if source.is_empty() || param.is_empty() || !value.is_finite() {
            return false;
        }
        queue_audio_set_param(self.command_queue.as_ref(), source, param, value as f32)
    }

    pub fn set_volume(&mut self, bus: &str, value: rhai::FLOAT) -> bool {
        if bus.is_empty() || !value.is_finite() {
            return false;
        }
        queue_audio_set_volume(self.command_queue.as_ref(), bus, value as f32)
    }
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<AudioApi>("WorldAudio")
        .register_fn("play", AudioApi::play)
        .register_fn("cue", AudioApi::cue)
        .register_fn("preload", AudioApi::preload)
        .register_fn("play_asset", AudioApi::play_asset)
        .register_fn("start_realtime", AudioApi::start_realtime)
        .register_fn("stop", AudioApi::stop)
        .register_fn("set_param", AudioApi::set_param)
        .register_fn("set_volume", AudioApi::set_volume);
}
