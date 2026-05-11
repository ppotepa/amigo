use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;

use crate::{
    RenderTargetInfo, RuntimeFrameInput, RuntimeFrameOutput, RuntimeSessionOptions,
    RuntimeSessionProfile, SceneSession,
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
