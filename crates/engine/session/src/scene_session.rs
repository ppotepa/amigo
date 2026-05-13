use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// High-level lifecycle state for the scene owned by a runtime session.
///
/// This is intentionally editor-facing and host-independent. It does not yet
/// replace app-owned scene handlers; it records the session-level lifecycle
/// boundary that later passes will move real scene loading/hydration/commands
/// behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneSessionLifecycleState {
    Empty,
    Loading,
    DocumentLoaded,
    HydrationQueued,
    Hydrated,
    TransitionPending,
    Clearing,
    Error,
}

impl Default for SceneSessionLifecycleState {
    fn default() -> Self {
        Self::Empty
    }
}

/// Host-independent scene session state.
///
/// Etap 4 keeps app-owned scene loading, hydration and handlers in place, but
/// gives the reusable session layer an explicit lifecycle model that app/editor
/// code can observe.
#[derive(Debug, Clone)]
pub struct SceneSession {
    lifecycle_state: SceneSessionLifecycleState,
    loaded_scene_document: Option<SceneSessionLoadedDocument>,
    hydration_queued_count: usize,
    applied_scene_command_count: usize,
    clear_count: usize,
    last_error: Option<String>,
}

/// Thread-safe runtime service wrapper for [`SceneSession`].
///
/// The app-owned scene command handlers still receive only the low-level runtime,
/// so the session lifecycle must be visible through the runtime service registry
/// during migration. `RuntimeSession` resolves the same service, so
/// game/app/editor-facing session state stays synchronized.
#[derive(Debug, Clone, Default)]
pub struct SceneSessionService {
    inner: Arc<Mutex<SceneSession>>,
}

impl SceneSessionService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> SceneSession {
        self.with_session(Clone::clone)
    }

    pub fn lifecycle_state(&self) -> SceneSessionLifecycleState {
        self.with_session(SceneSession::lifecycle_state)
    }

    pub fn lifecycle_summary(&self) -> SceneLifecycleSummary {
        self.with_session(SceneSession::lifecycle_summary)
    }

    pub fn active_scene_id(&self) -> Option<String> {
        self.with_session(|session| session.active_scene_id().map(str::to_owned))
    }

    pub fn begin_scene_load(&self, request: &SceneLoadRequest) -> SceneLifecycleSummary {
        self.with_session_mut(|session| session.begin_scene_load(request))
    }

    pub fn complete_scene_load(
        &self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLoadSummary {
        self.with_session_mut(|session| session.complete_scene_load(document))
    }

    pub fn fail_scene_load(
        &self,
        request: &SceneLoadRequest,
        error: impl Into<String>,
    ) -> SceneLifecycleSummary {
        self.with_session_mut(|session| session.fail_scene_load(request, error))
    }

    pub fn complete_hydration_queue(&self) -> SceneHydrationQueueSummary {
        self.with_session_mut(SceneSession::complete_hydration_queue)
    }

    pub fn apply_loaded_document(
        &self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLifecycleSummary {
        self.with_session_mut(|session| session.apply_loaded_document(document))
    }

    pub fn mark_loaded_scene_document(
        &self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLifecycleSummary {
        self.apply_loaded_document(document)
    }

    pub fn mark_hydration_queued(&self) -> SceneHydrationSummary {
        self.with_session_mut(SceneSession::mark_hydration_queued)
    }

    pub fn mark_scene_command_applied(&self) -> SceneCommandSummary {
        self.with_session_mut(SceneSession::mark_scene_command_applied)
    }

    pub fn mark_transition_pending(&self) -> SceneLifecycleSummary {
        self.with_session_mut(SceneSession::mark_transition_pending)
    }

    pub fn mark_clearing(&self) -> SceneLifecycleSummary {
        self.with_session_mut(SceneSession::mark_clearing)
    }

    pub fn mark_error(&self, error: impl Into<String>) -> SceneLifecycleSummary {
        self.with_session_mut(|session| session.mark_error(error))
    }

    pub fn clear_scene_metadata(&self) -> SceneClearSummary {
        self.with_session_mut(SceneSession::clear_scene_metadata)
    }

    fn with_session<T>(&self, f: impl FnOnce(&SceneSession) -> T) -> T {
        let guard = self.inner.lock().unwrap_or_else(|poison| poison.into_inner());
        f(&guard)
    }

    fn with_session_mut<T>(&self, f: impl FnOnce(&mut SceneSession) -> T) -> T {
        let mut guard = self.inner.lock().unwrap_or_else(|poison| poison.into_inner());
        f(&mut guard)
    }
}

impl Default for SceneSession {
    fn default() -> Self {
        Self {
            lifecycle_state: SceneSessionLifecycleState::Empty,
            loaded_scene_document: None,
            hydration_queued_count: 0,
            applied_scene_command_count: 0,
            clear_count: 0,
            last_error: None,
        }
    }
}

impl SceneSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current scene lifecycle state.
    pub fn lifecycle_state(&self) -> SceneSessionLifecycleState {
        self.lifecycle_state
    }

    /// Returns metadata for the loaded authored scene document, if any.
    pub fn loaded_scene_document(&self) -> Option<&SceneSessionLoadedDocument> {
        self.loaded_scene_document.as_ref()
    }

    /// Returns a host-independent lifecycle summary.
    pub fn lifecycle_summary(&self) -> SceneLifecycleSummary {
        SceneLifecycleSummary {
            state: self.lifecycle_state,
            active_scene_id: self.active_scene_id().map(str::to_owned),
            hydration_queued_count: self.hydration_queued_count,
            applied_scene_command_count: self.applied_scene_command_count,
            clear_count: self.clear_count,
            last_error: self.last_error.clone(),
        }
    }

    /// Returns the active loaded scene id, if known.
    pub fn active_scene_id(&self) -> Option<&str> {
        self.loaded_scene_document
            .as_ref()
            .map(|document| document.scene_id.as_str())
    }

    /// Mark the beginning of an app/session-driven scene load.
    pub fn begin_scene_load(&mut self, _request: &SceneLoadRequest) -> SceneLifecycleSummary {
        self.lifecycle_state = SceneSessionLifecycleState::Loading;
        self.loaded_scene_document = None;
        self.last_error = None;
        self.lifecycle_summary()
    }

    /// Complete a scene load and store authored document metadata.
    pub fn complete_scene_load(
        &mut self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLoadSummary {
        let lifecycle = self.apply_loaded_document(document.clone());

        SceneLoadSummary {
            loaded_scene: document,
            lifecycle,
        }
    }

    /// Mark a scene load failure.
    pub fn fail_scene_load(
        &mut self,
        request: &SceneLoadRequest,
        error: impl Into<String>,
    ) -> SceneLifecycleSummary {
        self.mark_error(format!(
            "failed to load scene `{}` from mod `{}`: {}",
            request.scene_id,
            request.mod_id,
            error.into()
        ))
    }

    /// Complete hydration queueing for the active scene document.
    pub fn complete_hydration_queue(&mut self) -> SceneHydrationQueueSummary {
        SceneHydrationQueueSummary {
            lifecycle: self.mark_hydration_queued(),
        }
    }

    /// Apply loaded authored scene metadata to the session.
    ///
    /// This is the Etap 4 replacement for directly mutating the loaded document
    /// field from app bootstrap code.
    pub fn apply_loaded_document(
        &mut self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLifecycleSummary {
        self.loaded_scene_document = Some(document);
        self.lifecycle_state = SceneSessionLifecycleState::DocumentLoaded;
        self.last_error = None;
        self.lifecycle_summary()
    }

    /// Mark a scene document as loaded by the session.
    ///
    /// Kept as a semantic alias for Etap 3 call sites. Prefer
    /// [`SceneSession::apply_loaded_document`] in new code.
    pub fn mark_loaded_scene_document(
        &mut self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLifecycleSummary {
        self.apply_loaded_document(document)
    }

    /// Mark that hydration commands were queued for the active document.
    pub fn mark_hydration_queued(&mut self) -> SceneHydrationSummary {
        self.hydration_queued_count += 1;
        self.lifecycle_state = SceneSessionLifecycleState::HydrationQueued;
        self.last_error = None;

        SceneHydrationSummary {
            state: self.lifecycle_state,
            active_scene_id: self.active_scene_id().map(str::to_owned),
            hydration_queued_count: self.hydration_queued_count,
        }
    }

    /// Mark that at least one scene command was applied.
    ///
    /// This does not replace command dispatch yet. It records lifecycle
    /// progress while app-owned command dispatch is still active.
    pub fn mark_scene_command_applied(&mut self) -> SceneCommandSummary {
        self.applied_scene_command_count += 1;
        if matches!(
            self.lifecycle_state,
            SceneSessionLifecycleState::DocumentLoaded
                | SceneSessionLifecycleState::HydrationQueued
        ) {
            self.lifecycle_state = SceneSessionLifecycleState::Hydrated;
        }
        self.last_error = None;

        SceneCommandSummary {
            state: self.lifecycle_state,
            active_scene_id: self.active_scene_id().map(str::to_owned),
            applied_scene_command_count: self.applied_scene_command_count,
        }
    }

    /// Mark that a scene transition is pending.
    pub fn mark_transition_pending(&mut self) -> SceneLifecycleSummary {
        self.lifecycle_state = SceneSessionLifecycleState::TransitionPending;
        self.last_error = None;
        self.lifecycle_summary()
    }

    /// Mark that scene-owned runtime content is being cleared.
    pub fn mark_clearing(&mut self) -> SceneLifecycleSummary {
        self.lifecycle_state = SceneSessionLifecycleState::Clearing;
        self.lifecycle_summary()
    }

    /// Mark a lifecycle error.
    pub fn mark_error(&mut self, error: impl Into<String>) -> SceneLifecycleSummary {
        self.lifecycle_state = SceneSessionLifecycleState::Error;
        self.last_error = Some(error.into());
        self.lifecycle_summary()
    }

    /// Clear session-owned scene metadata.
    ///
    /// This does not yet clear runtime scene services. That responsibility
    /// still lives in `amigo-app` until the scene lifecycle migration pass is
    /// completed.
    pub fn clear_loaded_scene_document(&mut self) -> SceneClearSummary {
        self.clear_scene_metadata()
    }

    /// Clear session-owned scene metadata and return a lifecycle summary.
    pub fn clear_scene_metadata(&mut self) -> SceneClearSummary {
        self.mark_clearing();
        self.loaded_scene_document = None;
        self.lifecycle_state = SceneSessionLifecycleState::Empty;
        self.clear_count += 1;
        self.last_error = None;

        SceneClearSummary {
            state: self.lifecycle_state,
            clear_count: self.clear_count,
        }
    }
}

/// Request to load an authored scene document.
///
/// Etap 5 uses this request as the session-level load lifecycle shape while the
/// concrete loader still delegates to app-owned scene runtime code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneLoadRequest {
    pub mod_id: String,
    pub scene_id: String,
}

impl SceneLoadRequest {
    pub fn new(mod_id: impl Into<String>, scene_id: impl Into<String>) -> Self {
        Self {
            mod_id: mod_id.into(),
            scene_id: scene_id.into(),
        }
    }
}

/// Summary returned when a scene document load completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneLoadSummary {
    pub loaded_scene: SceneSessionLoadedDocument,
    pub lifecycle: SceneLifecycleSummary,
}

/// Summary returned when hydration queueing completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneHydrationQueueSummary {
    pub lifecycle: SceneHydrationSummary,
}

/// Summary of an authored scene document loaded into a runtime session.
///
/// This mirrors only host-independent metadata. App/editor-specific summaries
/// should be converted into this type instead of being stored in
/// `amigo-session` directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSessionLoadedDocument {
    pub source_mod: String,
    pub scene_id: String,
    pub relative_path: PathBuf,
    pub entity_count: usize,
    pub component_count: usize,
    pub transition_count: usize,
}

impl SceneSessionLoadedDocument {
    pub fn new(
        source_mod: impl Into<String>,
        scene_id: impl Into<String>,
        relative_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            scene_id: scene_id.into(),
            relative_path: relative_path.into(),
            entity_count: 0,
            component_count: 0,
            transition_count: 0,
        }
    }

    pub fn with_counts(
        mut self,
        entity_count: usize,
        component_count: usize,
        transition_count: usize,
    ) -> Self {
        self.entity_count = entity_count;
        self.component_count = component_count;
        self.transition_count = transition_count;
        self
    }
}

/// High-level scene lifecycle summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneLifecycleSummary {
    pub state: SceneSessionLifecycleState,
    pub active_scene_id: Option<String>,
    pub hydration_queued_count: usize,
    pub applied_scene_command_count: usize,
    pub clear_count: usize,
    pub last_error: Option<String>,
}

/// Summary returned when scene hydration has been queued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneHydrationSummary {
    pub state: SceneSessionLifecycleState,
    pub active_scene_id: Option<String>,
    pub hydration_queued_count: usize,
}

/// Summary returned when a scene command has been applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneCommandSummary {
    pub state: SceneSessionLifecycleState,
    pub active_scene_id: Option<String>,
    pub applied_scene_command_count: usize,
}

/// Summary returned when scene metadata has been cleared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneClearSummary {
    pub state: SceneSessionLifecycleState,
    pub clear_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_session_starts_empty() {
        let session = SceneSession::new();

        assert_eq!(session.lifecycle_state(), SceneSessionLifecycleState::Empty);
        assert_eq!(session.active_scene_id(), None);
        assert!(session.loaded_scene_document().is_none());
    }

    #[test]
    fn loaded_document_changes_state_to_document_loaded() {
        let mut session = SceneSession::new();

        let summary = session.apply_loaded_document(SceneSessionLoadedDocument::new(
            "core",
            "main-menu",
            "scenes/main-menu.yaml",
        ));

        assert_eq!(summary.state, SceneSessionLifecycleState::DocumentLoaded);
        assert_eq!(session.active_scene_id(), Some("main-menu"));
    }

    #[test]
    fn begin_scene_load_changes_state_to_loading() {
        let mut session = SceneSession::new();
        let summary = session.begin_scene_load(&SceneLoadRequest::new("core", "main-menu"));

        assert_eq!(summary.state, SceneSessionLifecycleState::Loading);
        assert_eq!(summary.active_scene_id, None);
        assert!(summary.last_error.is_none());
    }

    #[test]
    fn complete_scene_load_records_loaded_document() {
        let mut session = SceneSession::new();
        let summary = session.complete_scene_load(SceneSessionLoadedDocument::new(
            "core",
            "main-menu",
            "scenes/main-menu.yaml",
        ));

        assert_eq!(summary.lifecycle.state, SceneSessionLifecycleState::DocumentLoaded);
        assert_eq!(summary.loaded_scene.scene_id, "main-menu");
        assert_eq!(session.active_scene_id(), Some("main-menu"));
    }

    #[test]
    fn fail_scene_load_enters_error_state() {
        let mut session = SceneSession::new();
        let summary = session.fail_scene_load(
            &SceneLoadRequest::new("core", "missing"),
            "not found",
        );

        assert_eq!(summary.state, SceneSessionLifecycleState::Error);
        assert!(summary.last_error.as_deref().is_some_and(|message| {
            message.contains("missing") && message.contains("not found")
        }));
    }

    #[test]
    fn hydration_queue_changes_state_to_hydration_queued() {
        let mut session = SceneSession::new();
        session.apply_loaded_document(SceneSessionLoadedDocument::new(
            "core",
            "main-menu",
            "scenes/main-menu.yaml",
        ));

        let summary = session.mark_hydration_queued();

        assert_eq!(summary.state, SceneSessionLifecycleState::HydrationQueued);
        assert_eq!(summary.active_scene_id.as_deref(), Some("main-menu"));
        assert_eq!(summary.hydration_queued_count, 1);
    }

    #[test]
    fn applied_scene_command_changes_hydration_queued_to_hydrated() {
        let mut session = SceneSession::new();
        session.apply_loaded_document(SceneSessionLoadedDocument::new(
            "core",
            "main-menu",
            "scenes/main-menu.yaml",
        ));
        session.mark_hydration_queued();

        let summary = session.mark_scene_command_applied();

        assert_eq!(summary.state, SceneSessionLifecycleState::Hydrated);
        assert_eq!(summary.applied_scene_command_count, 1);
    }

    #[test]
    fn scene_session_service_shares_state_between_clones() {
        let service = SceneSessionService::new();
        let clone = service.clone();

        service.apply_loaded_document(SceneSessionLoadedDocument::new(
            "core",
            "main-menu",
            "scenes/main-menu.yaml",
        ));

        assert_eq!(clone.active_scene_id().as_deref(), Some("main-menu"));
        assert_eq!(clone.lifecycle_state(), SceneSessionLifecycleState::DocumentLoaded);
    }

    #[test]
    fn clear_scene_metadata_returns_to_empty() {
        let mut session = SceneSession::new();
        session.apply_loaded_document(SceneSessionLoadedDocument::new(
            "core",
            "main-menu",
            "scenes/main-menu.yaml",
        ));
        session.mark_hydration_queued();

        let summary = session.clear_scene_metadata();

        assert_eq!(summary.state, SceneSessionLifecycleState::Empty);
        assert_eq!(summary.clear_count, 1);
        assert_eq!(session.active_scene_id(), None);
    }
}

