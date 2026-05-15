use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_beacon2d_set_aberration_px, queue_beacon2d_set_base_intensity,
    queue_beacon2d_set_beam_enabled, queue_beacon2d_set_beam_length_px,
    queue_beacon2d_set_beam_strength, queue_beacon2d_set_beam_width_degrees,
    queue_beacon2d_set_bloom, queue_beacon2d_set_core_radius_px, queue_beacon2d_set_duty_cycle,
    queue_beacon2d_set_flare_length_px, queue_beacon2d_set_flare_strength,
    queue_beacon2d_set_frequency_hz, queue_beacon2d_set_glow_strength,
    queue_beacon2d_set_halo_radius_px, queue_beacon2d_set_lens_influence,
};

#[derive(Clone)]
pub struct Beacon2dApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Beacon2dApi {
    pub fn set_base_intensity(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            12.0,
            queue_beacon2d_set_base_intensity,
        )
    }

    pub fn set_frequency_hz(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.02,
            12.0,
            queue_beacon2d_set_frequency_hz,
        )
    }

    pub fn set_duty_cycle(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.05,
            0.95,
            queue_beacon2d_set_duty_cycle,
        )
    }

    pub fn set_halo_radius_px(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            512.0,
            queue_beacon2d_set_halo_radius_px,
        )
    }

    pub fn set_core_radius_px(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            256.0,
            queue_beacon2d_set_core_radius_px,
        )
    }

    pub fn set_glow_strength(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            8.0,
            queue_beacon2d_set_glow_strength,
        )
    }

    pub fn set_beam_enabled(&mut self, target: &str, value: bool) -> bool {
        if target.is_empty() {
            return false;
        }
        queue_beacon2d_set_beam_enabled(self.command_queue.as_ref(), target, value)
    }

    pub fn set_beam_length_px(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            2048.0,
            queue_beacon2d_set_beam_length_px,
        )
    }

    pub fn set_beam_width_degrees(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            1.0,
            179.0,
            queue_beacon2d_set_beam_width_degrees,
        )
    }

    pub fn set_beam_strength(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            8.0,
            queue_beacon2d_set_beam_strength,
        )
    }

    pub fn set_aberration_px(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            32.0,
            queue_beacon2d_set_aberration_px,
        )
    }

    pub fn set_flare_length_px(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            2048.0,
            queue_beacon2d_set_flare_length_px,
        )
    }

    pub fn set_flare_strength(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            8.0,
            queue_beacon2d_set_flare_strength,
        )
    }

    pub fn set_bloom(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            8.0,
            queue_beacon2d_set_bloom,
        )
    }

    pub fn set_lens_influence(&mut self, target: &str, value: rhai::FLOAT) -> bool {
        queue_beacon2d_value(
            self.command_queue.as_ref(),
            target,
            value,
            0.0,
            8.0,
            queue_beacon2d_set_lens_influence,
        )
    }
}

fn queue_beacon2d_value(
    command_queue: Option<&Arc<ScriptCommandQueue>>,
    target: &str,
    value: rhai::FLOAT,
    min: f32,
    max: f32,
    queue: impl FnOnce(Option<&Arc<ScriptCommandQueue>>, &str, f32) -> bool,
) -> bool {
    if target.is_empty() || !value.is_finite() {
        return false;
    }
    queue(command_queue, target, (value as f32).clamp(min, max))
}
