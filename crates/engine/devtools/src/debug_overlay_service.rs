use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

use amigo_audio_output::AudioOutputBackendSnapshot;
use amigo_render_api::RenderFrameStats;
use amigo_session::SchedulingFrameStats;

use crate::{
    DebugOverlayAudioSnapshot, DebugOverlayCorner, DebugOverlayFrameSample,
    DebugOverlayInputSnapshot, DebugOverlayLayoutMode, DebugOverlayPanel,
    DebugOverlayParticleSnapshot, DebugOverlaySettings, DebugOverlaySnapshot,
};

const MAX_FRAME_HISTORY: usize = 240;

#[derive(Debug)]
struct DebugOverlayState {
    settings: DebugOverlaySettings,
    last_frame_instant: Option<Instant>,
    frame_history: VecDeque<DebugOverlayFrameSample>,
    latest_render_stats: RenderFrameStats,
    latest_scheduling_stats: SchedulingFrameStats,
    latest_audio_snapshot: DebugOverlayAudioSnapshot,
    latest_input_snapshot: DebugOverlayInputSnapshot,
    latest_particle_snapshot: DebugOverlayParticleSnapshot,
}

impl Default for DebugOverlayState {
    fn default() -> Self {
        Self {
            settings: DebugOverlaySettings::default(),
            last_frame_instant: None,
            frame_history: VecDeque::with_capacity(MAX_FRAME_HISTORY),
            latest_render_stats: RenderFrameStats::default(),
            latest_scheduling_stats: SchedulingFrameStats::default(),
            latest_audio_snapshot: DebugOverlayAudioSnapshot::default(),
            latest_input_snapshot: DebugOverlayInputSnapshot::default(),
            latest_particle_snapshot: DebugOverlayParticleSnapshot::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DebugOverlayService {
    state: Mutex<DebugOverlayState>,
}

impl DebugOverlayService {
    pub fn snapshot(&self) -> DebugOverlaySnapshot {
        let state = self
            .state
            .lock()
            .expect("debug overlay mutex should not be poisoned");

        DebugOverlaySnapshot {
            settings: state.settings.clone(),
            frame_history: state.frame_history.iter().cloned().collect(),
            render_stats: state.latest_render_stats.clone(),
            scheduling_stats: state.latest_scheduling_stats.clone(),
            audio: state.latest_audio_snapshot.clone(),
            input: state.latest_input_snapshot.clone(),
            particles: state.latest_particle_snapshot.clone(),
        }
    }

    pub fn record_render_frame(&self, stats: RenderFrameStats) {
        let now = Instant::now();
        let mut state = self
            .state
            .lock()
            .expect("debug overlay mutex should not be poisoned");

        let frame_ms = state
            .last_frame_instant
            .map(|previous| now.duration_since(previous).as_secs_f32() * 1000.0)
            .unwrap_or(0.0);
        let fps = if frame_ms > 0.0 {
            1000.0 / frame_ms
        } else {
            0.0
        };

        state.last_frame_instant = Some(now);
        state.latest_render_stats = stats.clone();
        state.frame_history.push_back(DebugOverlayFrameSample {
            frame_index: stats.frame_index,
            fps,
            frame_ms,
        });

        while state.frame_history.len() > MAX_FRAME_HISTORY {
            state.frame_history.pop_front();
        }
    }

    pub fn record_scheduling_stats(&self, stats: SchedulingFrameStats) {
        self.state
            .lock()
            .expect("debug overlay mutex should not be poisoned")
            .latest_scheduling_stats = stats;
    }

    pub fn record_audio_snapshot(
        &self,
        backend: AudioOutputBackendSnapshot,
        master_volume: f32,
        active_sources: usize,
        pending_commands: usize,
        bus_count: usize,
    ) {
        self.state
            .lock()
            .expect("debug overlay mutex should not be poisoned")
            .latest_audio_snapshot = DebugOverlayAudioSnapshot {
            backend_name: backend.backend_name,
            device_name: backend.device_name,
            sample_rate: backend.sample_rate,
            channels: backend.channels,
            started: backend.started,
            buffered_samples: backend.buffered_samples,
            last_error: backend.last_error,
            master_volume,
            active_sources,
            pending_commands,
            bus_count,
        };
    }

    pub fn record_input_snapshot(
        &self,
        backend_name: Option<String>,
        pressed_keys: Vec<String>,
        active_map: Option<String>,
        active_actions: Vec<String>,
    ) {
        self.state
            .lock()
            .expect("debug overlay mutex should not be poisoned")
            .latest_input_snapshot = DebugOverlayInputSnapshot {
            backend_name,
            pressed_keys,
            active_map,
            active_actions,
        };
    }

    pub fn record_particle_snapshot(&self, emitter_count: usize, active_emitters: usize) {
        self.state
            .lock()
            .expect("debug overlay mutex should not be poisoned")
            .latest_particle_snapshot = DebugOverlayParticleSnapshot {
            emitter_count,
            active_emitters,
        };
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.with_settings_mut(|settings| settings.enabled = enabled);
    }

    pub fn toggle_enabled(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("debug overlay mutex should not be poisoned");
        state.settings.enabled = !state.settings.enabled;
        state.settings.enabled
    }

    pub fn set_panel_visible(&self, panel: DebugOverlayPanel, visible: bool) {
        self.with_settings_mut(|settings| {
            if visible {
                settings.panels.insert(panel);
                settings.enabled = true;
            } else {
                settings.panels.remove(&panel);
            }
        });
    }

    pub fn toggle_panel(&self, panel: DebugOverlayPanel) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("debug overlay mutex should not be poisoned");

        let enabled = if state.settings.panels.contains(&panel) {
            state.settings.panels.remove(&panel);
            false
        } else {
            state.settings.panels.insert(panel);
            state.settings.enabled = true;
            true
        };

        enabled
    }

    pub fn set_layout_mode(&self, mode: DebugOverlayLayoutMode) {
        self.with_settings_mut(|settings| settings.layout_mode = mode);
    }

    pub fn set_scale(&self, scale: f32) {
        self.with_settings_mut(|settings| settings.scale = scale.clamp(0.5, 3.0));
    }

    pub fn set_corner(&self, corner: DebugOverlayCorner) {
        self.with_settings_mut(|settings| settings.corner = corner);
    }

    pub fn reset(&self) {
        let mut state = self
            .state
            .lock()
            .expect("debug overlay mutex should not be poisoned");
        state.settings = DebugOverlaySettings::default();
    }

    fn with_settings_mut(&self, f: impl FnOnce(&mut DebugOverlaySettings)) {
        let mut state = self
            .state
            .lock()
            .expect("debug overlay mutex should not be poisoned");
        f(&mut state.settings);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DebugOverlayCorner, DebugOverlayLayoutMode, DebugOverlayPanel, DebugOverlayService,
    };

    #[test]
    fn toggle_panel_enables_overlay() {
        let overlay = DebugOverlayService::default();

        assert!(overlay.toggle_panel(DebugOverlayPanel::Stats));
        let snapshot = overlay.snapshot();

        assert!(snapshot.settings.enabled);
        assert!(snapshot.settings.panels.contains(&DebugOverlayPanel::Stats));
    }

    #[test]
    fn reset_restores_defaults() {
        let overlay = DebugOverlayService::default();
        overlay.set_enabled(true);
        overlay.set_scale(2.5);
        overlay.set_corner(DebugOverlayCorner::BottomRight);
        overlay.set_layout_mode(DebugOverlayLayoutMode::Full);
        overlay.set_panel_visible(DebugOverlayPanel::Render, true);

        overlay.reset();
        let snapshot = overlay.snapshot();

        assert!(!snapshot.settings.enabled);
        assert_eq!(snapshot.settings.scale, 1.0);
        assert_eq!(snapshot.settings.corner, DebugOverlayCorner::TopLeft);
        assert_eq!(
            snapshot.settings.layout_mode,
            DebugOverlayLayoutMode::Compact
        );
        assert!(snapshot.settings.panels.contains(&DebugOverlayPanel::Fps));
        assert!(
            !snapshot
                .settings
                .panels
                .contains(&DebugOverlayPanel::Render)
        );
    }
}


