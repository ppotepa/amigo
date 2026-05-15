use amigo_render_api::RenderFrameStats;
use amigo_session::{RuntimeFrameClockSnapshot, SchedulingFrameStats};

use crate::DebugOverlaySettings;

#[derive(Debug, Clone, Default)]
pub struct DebugOverlayFrameSample {
    pub frame_index: u64,
    pub fps: f32,
    pub frame_ms: f32,
}

#[derive(Debug, Clone, Default)]
pub struct DebugOverlayAudioSnapshot {
    pub backend_name: String,
    pub device_name: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub started: bool,
    pub buffered_samples: usize,
    pub last_error: Option<String>,
    pub master_volume: f32,
    pub active_sources: usize,
    pub pending_commands: usize,
    pub bus_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct DebugOverlayInputSnapshot {
    pub backend_name: Option<String>,
    pub pressed_keys: Vec<String>,
    pub active_map: Option<String>,
    pub active_actions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DebugOverlayParticleSnapshot {
    pub emitter_count: usize,
    pub active_emitters: usize,
}

#[derive(Debug, Clone)]
pub struct DebugOverlaySnapshot {
    pub settings: DebugOverlaySettings,
    pub frame_history: Vec<DebugOverlayFrameSample>,
    pub render_stats: RenderFrameStats,
    pub scheduling_stats: SchedulingFrameStats,
    pub frame_clock: Option<RuntimeFrameClockSnapshot>,
    pub audio: DebugOverlayAudioSnapshot,
    pub input: DebugOverlayInputSnapshot,
    pub particles: DebugOverlayParticleSnapshot,
}
