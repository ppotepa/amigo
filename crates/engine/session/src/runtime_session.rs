use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::{Runtime, SystemPhase, SystemRegistry};

use crate::{
    RenderFrameErrorSummary, RenderFrameLifecycleSummary, RenderFrameSummary,
    RenderSessionLifecycleState, RenderSessionService, RenderTargetInfo, RuntimeCapabilityRegistry,
    RuntimeFrameInput, RuntimeFrameOutput, RuntimeSessionOptions, RuntimeSessionProfile,
    SceneClearSummary, SceneCommandSummary, SceneHydrationQueueSummary, SceneHydrationSummary,
    SceneLifecycleSummary, SceneLoadRequest, SceneLoadSummary, SceneSession,
    SceneSessionLifecycleState, SceneSessionLoadedDocument, SceneSessionService,
    SchedulerPhaseSummary, SchedulerSessionLifecycleState, SchedulerSessionService,
    ScriptCommandDispatchSummary, ScriptSessionLifecycleState, ScriptSessionService,
};

/// Reusable high-level runtime session.
pub struct RuntimeSession {
    runtime: Runtime,
    profile: RuntimeSessionProfile,
    scene_session: SceneSessionService,
    render_session: RenderSessionService,
    scheduler_session: SchedulerSessionService,
    script_session: ScriptSessionService,
    runtime_capabilities: RuntimeCapabilityRegistry,
}

impl RuntimeSession {
    pub fn from_runtime(runtime: Runtime, profile: RuntimeSessionProfile) -> Self {
        let scene_session = runtime
            .resolve::<SceneSessionService>()
            .map(|service| service.as_ref().clone())
            .unwrap_or_default();
        let render_session = runtime
            .resolve::<RenderSessionService>()
            .map(|service| service.as_ref().clone())
            .unwrap_or_default();
        let scheduler_session = runtime
            .resolve::<SchedulerSessionService>()
            .map(|service| service.as_ref().clone())
            .unwrap_or_default();
        let script_session = runtime
            .resolve::<ScriptSessionService>()
            .map(|service| service.as_ref().clone())
            .unwrap_or_default();

        Self {
            runtime,
            profile,
            scene_session,
            render_session,
            scheduler_session,
            script_session,
            runtime_capabilities: RuntimeCapabilityRegistry::new(),
        }
    }

    pub fn script_session_service(&self) -> &ScriptSessionService {
        &self.script_session
    }

    pub fn script_session_state(&self) -> ScriptSessionLifecycleState {
        self.script_session.lifecycle_state()
    }

    pub fn script_session_summary(&self) -> ScriptCommandDispatchSummary {
        self.script_session.script_dispatch_summary()
    }

    pub fn begin_script_command_dispatch(
        &self,
        command: impl Into<String>,
    ) -> ScriptCommandDispatchSummary {
        self.script_session.begin_script_command_dispatch(command)
    }

    pub fn complete_script_command_dispatch(&self) -> ScriptCommandDispatchSummary {
        self.script_session.complete_script_command_dispatch()
    }

    pub fn mark_script_dispatch_error(
        &self,
        command: impl Into<String>,
        error: impl Into<String>,
    ) -> ScriptCommandDispatchSummary {
        self.script_session
            .mark_script_dispatch_error(command, error)
    }

    pub fn scheduler_session_service(&self) -> &SchedulerSessionService {
        &self.scheduler_session
    }

    pub fn scheduler_session_state(&self) -> SchedulerSessionLifecycleState {
        self.scheduler_session.lifecycle_state()
    }

    pub fn scheduler_session_summary(&self) -> SchedulerPhaseSummary {
        self.scheduler_session.scheduler_summary()
    }

    pub fn begin_system_phase(&self, phase: SystemPhase) -> SchedulerPhaseSummary {
        self.scheduler_session.begin_system_phase(phase)
    }

    pub fn complete_system_phase(&self, phase: SystemPhase) -> SchedulerPhaseSummary {
        self.scheduler_session.complete_system_phase(phase)
    }

    pub fn mark_scheduler_error(
        &self,
        phase: SystemPhase,
        error: impl Into<String>,
    ) -> SchedulerPhaseSummary {
        self.scheduler_session.mark_error(phase, error)
    }

    pub fn run_phase(&self, phase: SystemPhase) -> AmigoResult<()> {
        let systems = self.runtime.resolve::<SystemRegistry>().ok_or_else(|| {
            AmigoError::Message("required service `SystemRegistry` is not registered".to_owned())
        })?;
        self.begin_system_phase(phase);
        if let Err(error) = systems.run_phase(phase, self.runtime()) {
            self.mark_scheduler_error(phase, format!("system phase {phase:?} failed: {error}"));
            return Err(error);
        }
        self.complete_system_phase(phase);
        Ok(())
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

    pub fn scene_session(&self) -> SceneSession {
        self.scene_session.snapshot()
    }

    pub fn scene_session_service(&self) -> &SceneSessionService {
        &self.scene_session
    }

    pub fn active_scene_id(&self) -> Option<String> {
        self.scene_session.active_scene_id()
    }

    pub fn scene_lifecycle_state(&self) -> SceneSessionLifecycleState {
        self.scene_session.lifecycle_state()
    }
    pub fn runtime_capabilities(&self) -> &crate::RuntimeCapabilityRegistry {
        &self.runtime_capabilities
    }

    pub fn runtime_capabilities_mut(&mut self) -> &mut crate::RuntimeCapabilityRegistry {
        &mut self.runtime_capabilities
    }

    pub fn runtime_contribution_summary(&self) -> crate::RuntimeCapabilitySummary {
        self.runtime_capabilities.summary()
    }

    pub fn runtime_diagnostics_summary(&self) -> crate::RuntimeCapabilityDiagnosticsSummary {
        crate::RuntimeCapabilityDiagnosticsSummary {
            descriptors: self
                .runtime_capabilities
                .descriptors_by_kind(crate::RuntimeCapabilityKind::DiagnosticsProvider)
                .map(|descriptor| crate::TargetAwareDiagnosticDescriptor {
                    descriptor: descriptor.clone(),
                    target: descriptor.domain_id.to_string(),
                })
                .collect(),
        }
    }

    pub fn runtime_metadata_summary(&self) -> crate::RuntimeCapabilityMetadataSummary {
        crate::RuntimeCapabilityMetadataSummary {
            descriptors: self
                .runtime_capabilities
                .descriptors_by_kind(crate::RuntimeCapabilityKind::MetadataProvider)
                .cloned()
                .collect(),
        }
    }

    pub fn scene_lifecycle_summary(&self) -> SceneLifecycleSummary {
        self.scene_session.lifecycle_summary()
    }

    pub fn begin_scene_load(&mut self, request: &SceneLoadRequest) -> SceneLifecycleSummary {
        self.scene_session.begin_scene_load(request)
    }

    pub fn complete_scene_load(
        &mut self,
        document: SceneSessionLoadedDocument,
    ) -> SceneLoadSummary {
        self.scene_session.complete_scene_load(document)
    }

    pub fn fail_scene_load(
        &mut self,
        request: &SceneLoadRequest,
        error: impl Into<String>,
    ) -> SceneLifecycleSummary {
        self.scene_session.fail_scene_load(request, error)
    }

    pub fn complete_scene_hydration_queue(&mut self) -> SceneHydrationQueueSummary {
        self.scene_session.complete_hydration_queue()
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

    pub fn mark_scene_lifecycle_error(
        &mut self,
        error: impl Into<String>,
    ) -> SceneLifecycleSummary {
        self.scene_session.mark_error(error)
    }

    pub fn mark_scene_clearing(&self) -> SceneLifecycleSummary {
        self.scene_session.mark_clearing()
    }

    pub fn clear_scene_metadata(&mut self) -> SceneClearSummary {
        self.scene_session.clear_scene_metadata()
    }

    pub fn render_session_service(&self) -> &RenderSessionService {
        &self.render_session
    }

    pub fn render_session(&self) -> crate::RenderSession {
        self.render_session.snapshot()
    }

    pub fn render_session_state(&self) -> RenderSessionLifecycleState {
        self.render_session.lifecycle_state()
    }

    pub fn begin_render_frame_extract(&self) -> RenderFrameLifecycleSummary {
        self.render_session.begin_frame_extract()
    }

    pub fn complete_render_frame_extract(&self) -> RenderFrameSummary {
        self.render_session.complete_frame_extract()
    }

    pub fn begin_render_composition(&self) -> RenderFrameLifecycleSummary {
        self.render_session.begin_composition()
    }

    pub fn complete_render_composition(&self) -> RenderFrameSummary {
        self.render_session.complete_composition()
    }

    pub fn complete_render_graph_build(&self) -> RenderFrameSummary {
        self.render_session.complete_graph_build()
    }

    pub fn complete_render_submit(&self) -> RenderFrameSummary {
        self.render_session.complete_submit()
    }

    pub fn complete_render_present(&self) -> RenderFrameSummary {
        self.render_session.complete_present()
    }

    pub fn mark_render_error(&self, error: impl Into<String>) -> RenderFrameErrorSummary {
        self.render_session.mark_error(error)
    }

    pub fn bootstrap(
        _options: RuntimeSessionOptions,
        _profile: RuntimeSessionProfile,
    ) -> AmigoResult<Self> {
        Err(AmigoError::Message(
            "RuntimeSession::bootstrap is intentionally not the active host bootstrap yet; amigo-app uses bootstrap_session_* as the P0.1 migration boundary. Move host-independent bootstrap here in a later domain migration."
                .to_owned(),
        ))
    }

    /// In P0.1 this remains a future host-independent API; active render flow is wired in app host wrappers.
    pub fn tick(&mut self, _input: RuntimeFrameInput) -> AmigoResult<RuntimeFrameOutput> {
        Ok(RuntimeFrameOutput::default())
    }

    /// In P0.1 this remains a future host-independent API; active render flow is wired in app host wrappers.
    pub fn build_render_request(&mut self, _target: RenderTargetInfo) -> AmigoResult<()> {
        Ok(())
    }
}
