//! Engine-owned loading telemetry and bounded preparation work.
//!
//! The service deliberately describes work in units rather than wall-clock
//! time. Domains therefore retain control over their own CPU/GPU preparation
//! while the host can advance a bounded amount of work every frame.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeLoadingPresentationMode {
    #[default]
    Overlay,
    Minimal,
    Hidden,
}

/// Resolved presentation settings. Colour values remain authored CSS-style
/// strings here; the UI composition bridge is responsible for rendering them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLoadingPresentation {
    pub enabled: bool,
    pub mode: RuntimeLoadingPresentationMode,
    pub title: String,
    pub show_stage: bool,
    pub show_item: bool,
    pub show_percentage: bool,
    pub background: Option<String>,
    pub accent: Option<String>,
    pub text: Option<String>,
}

impl Default for RuntimeLoadingPresentation {
    fn default() -> Self {
        Self { enabled: true, mode: RuntimeLoadingPresentationMode::Overlay, title: "Loading".into(), show_stage: true, show_item: true, show_percentage: true, background: None, accent: None, text: None }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuntimeLoadingPresentationPatch {
    pub enabled: Option<bool>,
    pub mode: Option<RuntimeLoadingPresentationMode>,
    pub title: Option<String>,
    pub show_stage: Option<bool>,
    #[serde(alias = "show_asset")]
    pub show_item: Option<bool>,
    pub show_percentage: Option<bool>,
    pub background: Option<String>,
    pub accent: Option<String>,
    pub text: Option<String>,
    pub theme: Option<RuntimeLoadingThemePatch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuntimeLoadingThemePatch {
    pub background: Option<String>,
    pub accent: Option<String>,
    pub text: Option<String>,
}

impl RuntimeLoadingPresentation {
    pub fn apply(&mut self, patch: RuntimeLoadingPresentationPatch) {
        if let Some(value) = patch.enabled { self.enabled = value; }
        if let Some(value) = patch.mode { self.mode = value; }
        if let Some(value) = patch.title { self.title = value; }
        if let Some(value) = patch.show_stage { self.show_stage = value; }
        if let Some(value) = patch.show_item { self.show_item = value; }
        if let Some(value) = patch.show_percentage { self.show_percentage = value; }
        if let Some(value) = patch.background { self.background = Some(value); }
        if let Some(value) = patch.accent { self.accent = Some(value); }
        if let Some(value) = patch.text { self.text = Some(value); }
        if let Some(theme) = patch.theme {
            if let Some(value) = theme.background { self.background = Some(value); }
            if let Some(value) = theme.accent { self.accent = Some(value); }
            if let Some(value) = theme.text { self.text = Some(value); }
        }
    }
}

/// Parses just the optional `[loading]` section from a mod manifest.
pub fn loading_presentation_from_mod_toml(source: &str) -> Result<RuntimeLoadingPresentationPatch, toml::de::Error> {
    #[derive(Deserialize)] struct Document { #[serde(default)] loading: RuntimeLoadingPresentationPatch }
    toml::from_str::<Document>(source).map(|document| document.loading)
}

/// Parses just the optional `loading:` section from a scene document.
pub fn loading_presentation_from_scene_yaml(source: &str) -> Result<RuntimeLoadingPresentationPatch, serde_yaml::Error> {
    #[derive(Deserialize)] struct Document { #[serde(default)] loading: RuntimeLoadingPresentationPatch }
    serde_yaml::from_str::<Document>(source).map(|document| document.loading)
}

pub fn resolve_loading_presentation(
    mod_patch: RuntimeLoadingPresentationPatch,
    scene_patch: RuntimeLoadingPresentationPatch,
) -> RuntimeLoadingPresentation {
    let mut resolved = RuntimeLoadingPresentation::default();
    resolved.apply(mod_patch);
    resolved.apply(scene_patch);
    resolved
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeLoadingState {
    #[default]
    Idle,
    Loading,
    Ready,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeLoadingSnapshot {
    pub state: RuntimeLoadingState,
    pub stage: String,
    pub current_item: Option<String>,
    pub completed_work_units: u64,
    pub total_work_units: u64,
    pub progress: f32,
    pub error: Option<String>,
    pub pending_jobs: usize,
    pub presentation: RuntimeLoadingPresentation,
}

impl Default for RuntimeLoadingSnapshot {
    fn default() -> Self {
        Self {
            state: RuntimeLoadingState::Idle,
            stage: String::new(),
            current_item: None,
            completed_work_units: 0,
            total_work_units: 0,
            progress: 0.0,
            error: None,
            pending_jobs: 0,
            presentation: RuntimeLoadingPresentation::default(),
        }
    }
}

impl RuntimeLoadingSnapshot {
    fn refresh_progress(&mut self) {
        self.progress = if self.total_work_units == 0 {
            if self.state == RuntimeLoadingState::Ready { 1.0 } else { 0.0 }
        } else {
            (self.completed_work_units as f32 / self.total_work_units as f32).clamp(0.0, 1.0)
        };
    }
}

/// Result returned by one incremental preparation slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeLoadingWorkResult {
    Pending { consumed_work_units: u64, current_item: Option<String> },
    Complete { consumed_work_units: u64 },
    Failed { consumed_work_units: u64, error: String },
}

/// Domain-owned incremental preparation. Implementations must consume no more
/// than `work_budget` units in one call.
pub trait RuntimeLoadingJob: Send {
    fn label(&self) -> &str;
    fn total_work_units(&self) -> u64;
    fn run_slice(&mut self, work_budget: u64) -> RuntimeLoadingWorkResult;
}

struct QueuedJob {
    job: Box<dyn RuntimeLoadingJob>,
}

#[derive(Default)]
struct RuntimeLoadingStateData {
    snapshot: RuntimeLoadingSnapshot,
    jobs: VecDeque<QueuedJob>,
}

/// Shared runtime service for loading lifecycle and preparation jobs.
///
/// It is a service rather than a renderer feature: presentation may be
/// suppressed without cancelling telemetry or preparation.
#[derive(Clone, Default)]
pub struct RuntimeLoadingService {
    inner: Arc<Mutex<RuntimeLoadingStateData>>,
}

impl std::fmt::Debug for RuntimeLoadingService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("RuntimeLoadingService").field("snapshot", &self.snapshot()).finish()
    }
}

impl RuntimeLoadingService {
    pub fn new() -> Self { Self::default() }

    pub fn snapshot(&self) -> RuntimeLoadingSnapshot {
        self.lock().snapshot.clone()
    }

    pub fn set_presentation(&self, presentation: RuntimeLoadingPresentation) {
        self.lock().snapshot.presentation = presentation;
    }

    pub fn begin(&self, stage: impl Into<String>, total_work_units: u64) {
        let mut state = self.lock();
        state.jobs.clear();
        let presentation = state.snapshot.presentation.clone();
        state.snapshot = RuntimeLoadingSnapshot {
            state: RuntimeLoadingState::Loading,
            stage: stage.into(),
            total_work_units,
            presentation,
            ..RuntimeLoadingSnapshot::default()
        };
        state.snapshot.refresh_progress();
    }

    pub fn set_stage(&self, stage: impl Into<String>, current_item: Option<String>) {
        let mut state = self.lock();
        if state.snapshot.state == RuntimeLoadingState::Idle {
            state.snapshot.state = RuntimeLoadingState::Loading;
        }
        state.snapshot.stage = stage.into();
        state.snapshot.current_item = current_item;
    }

    /// Adds externally performed work (scene parsing, asset IO, etc.).
    pub fn add_work(&self, total_work_units: u64) {
        let mut state = self.lock();
        if total_work_units > 0 && matches!(state.snapshot.state, RuntimeLoadingState::Idle | RuntimeLoadingState::Ready) {
            state.snapshot.state = RuntimeLoadingState::Loading;
        }
        state.snapshot.total_work_units = state.snapshot.total_work_units.saturating_add(total_work_units);
        state.snapshot.refresh_progress();
    }

    pub fn complete_work(&self, completed_work_units: u64) {
        let mut state = self.lock();
        state.snapshot.completed_work_units = state.snapshot.completed_work_units
            .saturating_add(completed_work_units)
            .min(state.snapshot.total_work_units);
        state.snapshot.refresh_progress();
    }

    pub fn enqueue(&self, job: impl RuntimeLoadingJob + 'static) {
        let mut state = self.lock();
        let units = job.total_work_units();
        if units > 0
            && matches!(
                state.snapshot.state,
                RuntimeLoadingState::Idle | RuntimeLoadingState::Ready
            )
        {
            state.snapshot.state = RuntimeLoadingState::Loading;
        }
        state.snapshot.total_work_units = state.snapshot.total_work_units.saturating_add(units);
        state.jobs.push_back(QueuedJob { job: Box::new(job) });
        state.snapshot.pending_jobs = state.jobs.len();
        state.snapshot.refresh_progress();
    }

    /// Advances queued jobs by at most `work_budget` aggregate units.
    pub fn run_frame(&self, work_budget: u64) -> RuntimeLoadingSnapshot {
        let mut state = self.lock();
        if state.snapshot.state != RuntimeLoadingState::Loading || work_budget == 0 {
            return state.snapshot.clone();
        }
        let mut remaining = work_budget;
        while remaining > 0 {
            let Some(mut queued) = state.jobs.pop_front() else { break };
            state.snapshot.current_item = Some(queued.job.label().to_owned());
            let result = queued.job.run_slice(remaining);
            let consumed = match &result {
                RuntimeLoadingWorkResult::Pending { consumed_work_units, .. }
                | RuntimeLoadingWorkResult::Complete { consumed_work_units }
                | RuntimeLoadingWorkResult::Failed { consumed_work_units, .. } => *consumed_work_units,
            }.min(remaining);
            state.snapshot.completed_work_units = state.snapshot.completed_work_units
                .saturating_add(consumed)
                .min(state.snapshot.total_work_units);
            remaining -= consumed;
            match result {
                RuntimeLoadingWorkResult::Pending { current_item, .. } => {
                    state.snapshot.current_item = current_item.or_else(|| Some(queued.job.label().to_owned()));
                    state.jobs.push_back(queued);
                    if consumed == 0 { break; }
                }
                RuntimeLoadingWorkResult::Complete { .. } => {}
                RuntimeLoadingWorkResult::Failed { error, .. } => {
                    state.snapshot.state = RuntimeLoadingState::Failed;
                    state.snapshot.error = Some(error);
                    state.jobs.clear();
                    break;
                }
            }
        }
        state.snapshot.pending_jobs = state.jobs.len();
        if state.snapshot.state == RuntimeLoadingState::Loading && state.jobs.is_empty()
            && state.snapshot.completed_work_units >= state.snapshot.total_work_units {
            state.snapshot.state = RuntimeLoadingState::Ready;
            state.snapshot.current_item = None;
        }
        state.snapshot.refresh_progress();
        state.snapshot.clone()
    }

    pub fn ready(&self) {
        let mut state = self.lock();
        state.jobs.clear();
        state.snapshot.pending_jobs = 0;
        state.snapshot.completed_work_units = state.snapshot.total_work_units;
        state.snapshot.current_item = None;
        state.snapshot.state = RuntimeLoadingState::Ready;
        state.snapshot.refresh_progress();
    }

    pub fn fail(&self, error: impl Into<String>) {
        let mut state = self.lock();
        state.jobs.clear();
        state.snapshot.pending_jobs = 0;
        state.snapshot.state = RuntimeLoadingState::Failed;
        state.snapshot.error = Some(error.into());
    }

    pub fn cancel(&self) {
        let mut state = self.lock();
        state.jobs.clear();
        state.snapshot.pending_jobs = 0;
        state.snapshot.current_item = None;
        state.snapshot.state = RuntimeLoadingState::Cancelled;
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RuntimeLoadingStateData> {
        self.inner.lock().unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Units(u64);
    impl RuntimeLoadingJob for Units {
        fn label(&self) -> &str { "prepare mesh" }
        fn total_work_units(&self) -> u64 { self.0 }
        fn run_slice(&mut self, budget: u64) -> RuntimeLoadingWorkResult {
            let consumed = budget.min(self.0);
            self.0 -= consumed;
            if self.0 == 0 { RuntimeLoadingWorkResult::Complete { consumed_work_units: consumed } }
            else { RuntimeLoadingWorkResult::Pending { consumed_work_units: consumed, current_item: None } }
        }
    }

    #[test]
    fn work_is_bounded_per_frame_and_reaches_ready() {
        let service = RuntimeLoadingService::new();
        service.begin("Preparing", 0);
        service.enqueue(Units(5));
        assert_eq!(service.run_frame(2).completed_work_units, 2);
        assert_eq!(service.run_frame(2).completed_work_units, 4);
        let snapshot = service.run_frame(2);
        assert_eq!(snapshot.state, RuntimeLoadingState::Ready);
        assert_eq!(snapshot.progress, 1.0);
    }

    #[test]
    fn domain_work_after_hydration_reopens_loading_until_completed() {
        let service = RuntimeLoadingService::new();
        service.begin("Hydration", 1);
        service.ready();
        service.add_work(2);
        service.set_stage("Preparing surfaces", Some("building.glb".into()));
        assert_eq!(service.run_frame(4).state, RuntimeLoadingState::Loading);
        service.complete_work(1);
        assert_eq!(service.run_frame(4).state, RuntimeLoadingState::Loading);
        service.complete_work(1);
        assert_eq!(service.run_frame(4).state, RuntimeLoadingState::Ready);
    }

    #[test]
    fn queued_work_after_ready_reopens_loading() {
        let service = RuntimeLoadingService::new();
        service.begin("Hydration", 1);
        service.ready();
        service.enqueue(Units(1));
        assert_eq!(service.snapshot().state, RuntimeLoadingState::Loading);
        assert_eq!(service.run_frame(1).state, RuntimeLoadingState::Ready);
    }

    #[test]
    fn presentation_merges_mod_then_scene_and_can_hide_only_the_overlay() {
        let mod_patch = loading_presentation_from_mod_toml("[loading]\ntitle = 'Preparing city'\naccent = '#F6B44C'").unwrap();
        let scene_patch = loading_presentation_from_scene_yaml("loading:\n  enabled: false\n  theme:\n    background: '#101827E8'").unwrap();
        let resolved = resolve_loading_presentation(mod_patch, scene_patch);
        assert!(!resolved.enabled);
        assert_eq!(resolved.title, "Preparing city");
        assert_eq!(resolved.background.as_deref(), Some("#101827E8"));
    }
}
