use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_math::Vec2;

use crate::{BeaconLight2dCommand, BeaconLight2dDrawCommand};

#[derive(Debug, Default)]
pub struct BeaconLight2dSceneService {
    state: Mutex<BeaconState>,
}

#[derive(Debug, Default)]
struct BeaconState {
    beacons: Vec<BeaconLight2dCommand>,
    time_seconds: f32,
}

impl BeaconLight2dSceneService {
    pub fn queue(&self, command: BeaconLight2dCommand) {
        let mut state = self.state.lock().expect("beacon state mutex");
        state.beacons.retain(|b| b.id != command.id);
        state.beacons.push(command);
    }

    pub fn clear(&self) {
        let mut state = self.state.lock().expect("beacon state mutex");
        state.beacons.clear();
        state.time_seconds = 0.0;
    }

    pub fn commands(&self) -> Vec<BeaconLight2dCommand> {
        self.state
            .lock()
            .expect("beacon state mutex")
            .beacons
            .clone()
    }

    pub fn set_base_intensity(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.base_intensity = value.clamp(0.0, 12.0);
        })
    }

    pub fn set_frequency_hz(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.frequency_hz = value.clamp(0.02, 12.0);
        })
    }

    pub fn set_duty_cycle(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.duty_cycle = value.clamp(0.05, 0.95);
        })
    }

    pub fn set_halo_radius_px(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.halo_radius_px = value.clamp(0.0, 512.0);
        })
    }

    pub fn set_aberration_px(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.aberration_px = value.clamp(0.0, 32.0);
        })
    }

    pub fn set_flare_strength(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.flare_strength = value.clamp(0.0, 8.0);
        })
    }

    pub fn tick(&self, delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        let mut state = self.state.lock().expect("beacon state mutex");
        state.time_seconds += delta_seconds;
    }

    pub fn draw_commands(&self) -> Vec<BeaconLight2dDrawCommand> {
        let state = self.state.lock().expect("beacon state mutex");
        let sync_phase = build_sync_group_phase_map(&state.beacons);
        state
            .beacons
            .iter()
            .filter(|b| b.enabled)
            .map(|b| {
                let base_phase = b
                    .sync_group
                    .as_ref()
                    .and_then(|g| sync_phase.get(g))
                    .copied()
                    .unwrap_or_else(|| seeded_unit(&b.id));
                let phase =
                    fract(state.time_seconds * b.frequency_hz + b.phase_offset + base_phase);
                let gate = pulse_gate(
                    phase,
                    b.duty_cycle,
                    b.rise_seconds,
                    b.fall_seconds,
                    b.frequency_hz,
                );
                let jitter_phase = b
                    .sync_group
                    .as_ref()
                    .map(|g| format!("{g}:jitter"))
                    .unwrap_or_else(|| format!("{}:jitter", b.id));
                let jitter_seed = seeded_unit(&jitter_phase);
                let jitter = 1.0
                    + b.jitter_amount
                        * ((state.time_seconds * b.jitter_hz + jitter_seed)
                            * std::f32::consts::TAU)
                            .sin();
                let intensity =
                    (b.base_intensity * (0.10 + gate * 0.90) * jitter.max(0.0)).max(0.0);
                BeaconLight2dDrawCommand {
                    entity_name: b.entity_name.clone(),
                    render_layer: b.render_layer.clone(),
                    z_index: b.z_index,
                    center: Vec2::new(b.transform.translation.x, b.transform.translation.y),
                    color: b.color,
                    intensity,
                    core_radius_px: b.core_radius_px,
                    halo_radius_px: b.halo_radius_px,
                    aberration_px: b.aberration_px,
                    flare_length_px: b.flare_length_px,
                    flare_strength: b.flare_strength,
                    viewport_fit: b.viewport_fit,
                    viewport_canvas_size: b.viewport_canvas_size,
                }
            })
            .collect()
    }

    fn update_target(
        &self,
        target: &str,
        mut update: impl FnMut(&mut BeaconLight2dCommand),
    ) -> bool {
        if target.is_empty() {
            return false;
        }
        let mut state = self.state.lock().expect("beacon state mutex");
        let all = matches!(target, "*" | "all");
        let mut updated = false;
        for beacon in &mut state.beacons {
            if all || beacon.id == target || beacon.entity_name == target {
                update(beacon);
                updated = true;
            }
        }
        updated
    }
}

fn build_sync_group_phase_map(beacons: &[BeaconLight2dCommand]) -> BTreeMap<String, f32> {
    let mut map = BTreeMap::new();
    for beacon in beacons {
        if let Some(group) = beacon.sync_group.as_ref() {
            map.entry(group.clone())
                .or_insert_with(|| seeded_unit(group));
        }
    }
    map
}

fn fract(v: f32) -> f32 {
    v - v.floor()
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn pulse_gate(
    phase: f32,
    duty: f32,
    rise_seconds: f32,
    fall_seconds: f32,
    frequency_hz: f32,
) -> f32 {
    let duty = duty.clamp(0.05, 0.95);
    let period = if frequency_hz > 0.001 {
        1.0 / frequency_hz
    } else {
        1.0
    };
    let rise_phase = (rise_seconds / period).clamp(0.001, duty * 0.75);
    let fall_phase = (fall_seconds / period).clamp(0.001, 1.0 - duty);
    let rise = smoothstep(0.0, rise_phase, phase);
    let fall = 1.0 - smoothstep(duty, duty + fall_phase, phase);
    (rise * fall).clamp(0.0, 1.0)
}

fn seeded_unit(seed: &str) -> f32 {
    let mut hash: u64 = 1469598103934665603;
    for b in seed.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    (hash as f64 / u64::MAX as f64) as f32
}
