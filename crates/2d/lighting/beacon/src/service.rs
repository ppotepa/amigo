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
            beacon.halo_radius_px = value.clamp(0.0, 1100.0);
        })
    }

    pub fn set_core_radius_px(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.core_radius_px = value.clamp(0.0, 128.0);
        })
    }

    pub fn set_glow_strength(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.glow_strength = value.clamp(0.0, 8.0);
        })
    }

    pub fn set_beam_enabled(&self, target: &str, value: bool) -> bool {
        self.update_target(target, |beacon| {
            beacon.beam_enabled = value;
        })
    }

    pub fn set_beam_length_px(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.beam_length_px = value.clamp(0.0, 2048.0);
        })
    }

    pub fn set_beam_width_degrees(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.beam_width_degrees = value.clamp(1.0, 179.0);
        })
    }

    pub fn set_beam_strength(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.beam_strength = value.clamp(0.0, 8.0);
        })
    }

    pub fn set_aberration_px(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.aberration_px = value.clamp(0.0, 32.0);
        })
    }

pub fn set_bloom(&self, target: &str, value: f32) -> bool {
        self.update_target(target, |beacon| {
            beacon.bloom = value.clamp(0.0, 8.0);
        })
    }
pub fn set_position_2d(&self, target: &str, x: f32, y: f32) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        self.update_target(target, |beacon| {
            beacon.transform.translation.x = x;
            beacon.transform.translation.y = y;
        })
    }

    pub fn set_distance_m(&self, target: &str, distance_m: f32) -> bool {
        if !distance_m.is_finite() {
            return false;
        }
        self.update_target(target, |beacon| {
            beacon.distance_m = Some(distance_m.clamp(0.1, 250.0));
        })
    }

    pub fn set_z_depth(&self, target: &str, z_depth: f32) -> bool {
        if !z_depth.is_finite() {
            return false;
        }
        self.update_target(target, |beacon| {
            beacon.z_depth = Some(z_depth.clamp(0.0, 1.0));
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
                let gate = pulse_gate(phase, b.duty_cycle);
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
                let pulse = 0.36 + 0.64 * gate;
                let intensity = (b.base_intensity * pulse * jitter.max(0.0)).max(0.0);
                BeaconLight2dDrawCommand {
                    entity_name: b.entity_name.clone(),
                    render_layer: b.render_layer.clone(),
                    z_index: b.z_index,
                    center: Vec2::new(b.transform.translation.x, b.transform.translation.y),
                    color: b.color,
                    intensity,
                    pulse,
                    core_radius_px: b.core_radius_px,
                    halo_radius_px: b.halo_radius_px,
                    glow_strength: b.glow_strength,
                    rotation_radians: b.transform.rotation_radians,
                    beam_enabled: b.beam_enabled,
                    beam_length_px: b.beam_length_px,
                    beam_width_degrees: b.beam_width_degrees,
                    beam_strength: b.beam_strength,
                    aberration_px: b.aberration_px,
                    
                    
                    bloom: b.bloom,
                    camera_response: b.camera_response,
                    distance_m: b.distance_m,
                    z_depth: b.z_depth,
                    render_contributions: b.render_contributions.clone(),
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

fn pulse_gate(phase: f32, duty: f32) -> f32 {
    let duty = duty.clamp(0.05, 0.95);
    let wave = 0.5 + 0.5 * (phase * std::f32::consts::TAU).sin();
    smoothstep(1.0 - duty, 1.0, wave)
}

fn seeded_unit(seed: &str) -> f32 {
    let mut hash: u64 = 1469598103934665603;
    for b in seed.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    (hash as f64 / u64::MAX as f64) as f32
}
