use crate::render_runtime::RenderFrameStats;
use crate::scheduling::SchedulingFrameStats;

use super::service::DebugOverlaySettings;

#[derive(Debug, Clone, Default)]
pub(crate) struct DebugOverlayFrameSample {
    pub(crate) frame_index: u64,
    pub(crate) fps: f32,
    pub(crate) frame_ms: f32,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DebugOverlayAudioSnapshot {
    pub(crate) backend_name: String,
    pub(crate) device_name: Option<String>,
    pub(crate) sample_rate: Option<u32>,
    pub(crate) channels: Option<u16>,
    pub(crate) started: bool,
    pub(crate) buffered_samples: usize,
    pub(crate) last_error: Option<String>,
    pub(crate) master_volume: f32,
    pub(crate) active_sources: usize,
    pub(crate) pending_commands: usize,
    pub(crate) bus_count: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DebugOverlayInputSnapshot {
    pub(crate) backend_name: Option<String>,
    pub(crate) pressed_keys: Vec<String>,
    pub(crate) active_map: Option<String>,
    pub(crate) active_actions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DebugOverlayParticleSnapshot {
    pub(crate) emitter_count: usize,
    pub(crate) active_emitters: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct DebugOverlaySnapshot {
    pub(crate) settings: DebugOverlaySettings,
    pub(crate) frame_history: Vec<DebugOverlayFrameSample>,
    pub(crate) render_stats: RenderFrameStats,
    pub(crate) scheduling_stats: SchedulingFrameStats,
    pub(crate) audio: DebugOverlayAudioSnapshot,
    pub(crate) input: DebugOverlayInputSnapshot,
    pub(crate) particles: DebugOverlayParticleSnapshot,
}
