use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_audio_play, queue_audio_play_asset, queue_audio_set_param, queue_audio_set_volume,
    queue_audio_start_realtime, queue_audio_stop,
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
