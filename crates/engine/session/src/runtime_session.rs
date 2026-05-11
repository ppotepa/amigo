use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;

use crate::{
    RenderTargetInfo, RuntimeFrameInput, RuntimeFrameOutput, RuntimeSessionOptions,
    RuntimeSessionProfile, SceneClearSummary, SceneCommandSummary, SceneHydrationSummary,
    SceneLifecycleSummary, SceneSession, SceneSessionLifecycleState,
    SceneSessionLoadedDocument,
};

/// Reusable high-level runtime session.
pub struct RuntimeSession {
    runtime: Runtime,
    profile: RuntimeSessionProfile,
    scene_session: SceneSession,
}

impl RuntimeSession {
    pub fn from_runtime(runtime: Runtime, profile: RuntimeSessionProfile) -> Self {
        Self {
            runtime,
            profile,
            scene_session: SceneSession::new(),
        }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    /// Consume the session and return the underlying low-level runtime.
    pub fn into_runtime(self) -> Runtime {
        self.runtime
    }

    pub fn profile(&self) -> RuntimeSessionProfile {
        self.profile
    }

    pub fn scene_session(&self) -> &SceneSession {
        &self.scene_session
    }

    pub fn scene_session_mut(&mut self) -> &mut SceneSession {
        &mut self.scene_session
    }

    pub fn active_scene_id(&self) -> Option<&str> {
        self.scene_session.active_scene_id()
    }

    pub fn scene_lifecycle_state(&self) -> SceneSessionLifecycleState {
        self.scene_session.lifecycle_state()
    }

    pub fn scene_lifecycle_summary(&self) -> SceneLifecycleSummary {
        self.scene_session.lifecycle_summary()
    }

    pub fn mark_scene_loaded(
        &mut self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLifecycleSummary {
        self.scene_session.apply_loaded_document(document)
    }

    pub fn mark_scene_hydration_queued(&mut self) -> SceneHydrationSummary {
        self.scene_session.mark_hydration_queued()
    }

    pub fn mark_scene_command_applied(&mut self) -> SceneCommandSummary {
        self.scene_session.mark_scene_command_applied()
    }

    pub fn clear_scene_metadata(&mut self) -> SceneClearSummary {
        self.scene_session.clear_scene_metadata()
    }

    pub fn bootstrap(
        _options: RuntimeSessionOptions,
        _profile: RuntimeSessionProfile,
    ) -> AmigoResult<Self> {
        Err(AmigoError::Message(
            "RuntimeSession::bootstrap is not migrated yet; move app bootstrap in Etap 2"
                .to_owned(),
        ))
    }

    pub fn tick(&mut self, _input: RuntimeFrameInput) -> AmigoResult<RuntimeFrameOutput> {
        Ok(RuntimeFrameOutput::default())
    }

    pub fn build_render_request(&mut self, _target: RenderTargetInfo) -> AmigoResult<()> {
        Ok(())
    }
}
