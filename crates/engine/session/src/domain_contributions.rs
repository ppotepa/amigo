use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeDomainId {
    value: String,
}

impl RuntimeDomainId {
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for RuntimeDomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

pub const APP_LEGACY_DOMAIN_ID: &str = "app.legacy";
pub const APP_HOST_DOMAIN_ID: &str = "app.host";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeContributionKind {
    SceneCommandHandler,
    ScriptCommandHandler,
    SystemPhaseHandler,
    RenderExtractor,
    DevConsoleCommand,
    DiagnosticsProvider,
    MetadataProvider,
}

#[derive(Debug, Clone)]
pub struct RuntimeContributionDescriptor {
    pub domain_id: RuntimeDomainId,
    pub kind: RuntimeContributionKind,
    pub id: String,
    pub label: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub migration_seam: bool,
}

impl RuntimeContributionDescriptor {
    pub fn is_app_legacy(&self) -> bool {
        self.domain_id.as_str() == APP_LEGACY_DOMAIN_ID
    }

    pub fn is_app_host(&self) -> bool {
        self.domain_id.as_str() == APP_HOST_DOMAIN_ID
    }

    pub fn is_domain_owned(&self) -> bool {
        !self.is_app_legacy() && !self.is_app_host()
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeDomainContribution {
    pub descriptor: RuntimeContributionDescriptor,
}

#[derive(Debug, Clone)]
pub struct DevConsoleCommandDescriptor {
    pub descriptor: RuntimeContributionDescriptor,
}

#[derive(Debug, Clone)]
pub struct DevConsoleCommandContribution {
    pub descriptor: DevConsoleCommandDescriptor,
}

pub trait DevConsoleCommandProvider {
    fn register_dev_console_commands(&self, session_descriptors: &mut Vec<DevConsoleCommandDescriptor>);
}

#[derive(Debug, Clone)]
pub struct DiagnosticsProviderContribution {
    pub descriptor: TargetAwareDiagnosticDescriptor,
}

#[derive(Debug, Clone)]
pub struct TargetAwareDiagnosticDescriptor {
    pub descriptor: RuntimeContributionDescriptor,
    pub target: String,
}

pub trait DiagnosticsProvider {
    fn register_diagnostics(&self, session_descriptors: &mut Vec<TargetAwareDiagnosticDescriptor>);
}

#[derive(Debug, Clone)]
pub struct MetadataProviderContribution {
    pub descriptor: RuntimeContributionDescriptor,
}

pub trait MetadataProvider {
    fn register_metadata(&self, session_descriptors: &mut Vec<RuntimeContributionDescriptor>);
}

#[derive(Debug, Clone)]
pub struct ScriptCommandHandlerDescriptor {
    pub descriptor: RuntimeContributionDescriptor,
    pub handler_id: String,
}

#[derive(Debug, Clone)]
pub struct ScriptCommandDispatchContext {
    pub command_label: String,
}

#[derive(Debug, Clone)]
pub struct ScriptCommandDispatchResult {
    pub handled: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScriptCommandHandlerContribution {
    pub descriptor: ScriptCommandHandlerDescriptor,
}

pub trait ScriptCommandProvider {
    fn register_script_command_handlers(&self, session_descriptors: &mut Vec<ScriptCommandHandlerDescriptor>);
}

#[derive(Debug, Clone)]
pub struct SceneCommandHandlerDescriptor {
    pub descriptor: RuntimeContributionDescriptor,
    pub handler_id: String,
}

#[derive(Debug, Clone)]
pub struct SceneCommandDispatchContext {
    pub command_label: String,
}

#[derive(Debug, Clone)]
pub struct SceneCommandDispatchResult {
    pub handled: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SceneCommandHandlerContribution {
    pub descriptor: SceneCommandHandlerDescriptor,
}

pub trait SceneCommandProvider {
    fn register_scene_command_handlers(&self, session_descriptors: &mut Vec<SceneCommandHandlerDescriptor>);
}

#[derive(Debug, Clone)]
pub struct SystemDescriptor {
    pub domain_id: RuntimeDomainId,
    pub system_id: String,
    pub phase: String,
    pub ordering: usize,
    pub main_thread_required: bool,
    pub diagnostics_label: String,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub migration_seam: bool,
}

impl SystemDescriptor {
    pub fn is_app_legacy(&self) -> bool {
        self.domain_id.as_str() == APP_LEGACY_DOMAIN_ID
    }

    pub fn is_app_host(&self) -> bool {
        self.domain_id.as_str() == APP_HOST_DOMAIN_ID
    }

    pub fn is_domain_owned(&self) -> bool {
        !self.is_app_legacy() && !self.is_app_host()
    }
}

#[derive(Debug, Clone)]
pub struct SystemContribution {
    pub descriptor: SystemDescriptor,
}

#[derive(Debug, Clone)]
pub struct SystemRunContext {
    pub phase: String,
    pub system_id: String,
}

#[derive(Debug, Clone)]
pub struct SystemRunResult {
    pub handled: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SystemPhaseContribution {
    pub descriptor: SystemDescriptor,
}

pub trait SystemProvider {
    fn register_system_phase_contributions(
        &self,
        session_descriptors: &mut Vec<SystemDescriptor>,
    );
}

#[derive(Debug, Clone)]
pub struct RenderExtractorDescriptor {
    pub descriptor: RuntimeContributionDescriptor,
}

#[derive(Debug, Clone)]
pub struct RenderExtractContext {
    // Placeholder for future render-extractor execution inputs.
}

#[derive(Debug, Clone)]
pub struct RenderExtractOutput {
    // Placeholder for future render-extractor execution outputs.
}

#[derive(Debug, Clone)]
pub struct RenderExtractorContribution {
    pub descriptor: RenderExtractorDescriptor,
}

pub trait RenderExtractorProvider {
    fn register_render_extractors(&self, session_descriptors: &mut Vec<RenderExtractorDescriptor>);
}

#[derive(Debug, Clone)]
pub struct RuntimeContributionSummary {
    pub total: usize,
    pub by_kind: BTreeMap<RuntimeContributionKind, usize>,
}

#[derive(Debug, Clone)]
pub struct RuntimeContributionDiagnosticsSummary {
    pub descriptors: Vec<TargetAwareDiagnosticDescriptor>,
}

#[derive(Debug, Clone)]
pub struct RuntimeContributionMetadataSummary {
    pub descriptors: Vec<RuntimeContributionDescriptor>,
}

#[derive(Debug, Default)]
pub struct RuntimeDomainContributionRegistry {
    contributions: Vec<RuntimeDomainContribution>,
}

impl RuntimeDomainContributionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, contribution: RuntimeDomainContribution) {
        self.contributions.push(contribution);
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &RuntimeContributionDescriptor> {
        self.contributions.iter().map(|entry| &entry.descriptor)
    }

    pub fn descriptors_by_kind(
        &self,
        kind: RuntimeContributionKind,
    ) -> impl Iterator<Item = &RuntimeContributionDescriptor> {
        self.contributions
            .iter()
            .map(|entry| &entry.descriptor)
            .filter(move |descriptor| descriptor.kind == kind)
    }

    pub fn descriptors_by_domain(
        &self,
        domain_id: &str,
    ) -> impl Iterator<Item = &RuntimeContributionDescriptor> {
        self.contributions
            .iter()
            .map(|entry| &entry.descriptor)
            .filter(move |descriptor| descriptor.domain_id.as_str() == domain_id)
    }

    pub fn count_app_legacy(&self) -> usize {
        self.descriptors()
            .filter(|descriptor| descriptor.is_app_legacy())
            .count()
    }

    pub fn count_app_host(&self) -> usize {
        self.descriptors()
            .filter(|descriptor| descriptor.is_app_host())
            .count()
    }

    pub fn summary(&self) -> RuntimeContributionSummary {
        let mut by_kind = BTreeMap::new();
        for descriptor in self.descriptors() {
            *by_kind.entry(descriptor.kind).or_insert(0) += 1;
        }

        RuntimeContributionSummary {
            total: self.contributions.len(),
            by_kind,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.contributions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(
        domain_id: &str,
        kind: RuntimeContributionKind,
        id: &str,
    ) -> RuntimeContributionDescriptor {
        RuntimeContributionDescriptor {
            domain_id: RuntimeDomainId::new(domain_id),
            kind,
            id: id.to_string(),
            label: id.to_string(),
            description: format!("{id} description"),
            capabilities: Vec::new(),
            tags: Vec::new(),
            migration_seam: domain_id == APP_LEGACY_DOMAIN_ID,
        }
    }

    #[test]
    fn runtime_contribution_descriptor_classifies_ownership() {
        let legacy = descriptor(
            APP_LEGACY_DOMAIN_ID,
            RuntimeContributionKind::SceneCommandHandler,
            "legacy",
        );
        let host = descriptor(
            APP_HOST_DOMAIN_ID,
            RuntimeContributionKind::RenderExtractor,
            "host",
        );
        let domain = descriptor(
            "amigo.2d.vector",
            RuntimeContributionKind::SceneCommandHandler,
            "vector",
        );

        assert!(legacy.is_app_legacy());
        assert!(!legacy.is_app_host());
        assert!(!legacy.is_domain_owned());

        assert!(!host.is_app_legacy());
        assert!(host.is_app_host());
        assert!(!host.is_domain_owned());

        assert!(!domain.is_app_legacy());
        assert!(!domain.is_app_host());
        assert!(domain.is_domain_owned());
    }

    #[test]
    fn registry_counts_and_filters_by_domain() {
        let mut registry = RuntimeDomainContributionRegistry::new();
        registry.register(RuntimeDomainContribution {
            descriptor: descriptor(
                APP_LEGACY_DOMAIN_ID,
                RuntimeContributionKind::SceneCommandHandler,
                "legacy.scene",
            ),
        });
        registry.register(RuntimeDomainContribution {
            descriptor: descriptor(
                APP_LEGACY_DOMAIN_ID,
                RuntimeContributionKind::RenderExtractor,
                "legacy.render",
            ),
        });
        registry.register(RuntimeDomainContribution {
            descriptor: descriptor(
                APP_HOST_DOMAIN_ID,
                RuntimeContributionKind::RenderExtractor,
                "host.overlay",
            ),
        });
        registry.register(RuntimeDomainContribution {
            descriptor: descriptor(
                "amigo.2d.vector",
                RuntimeContributionKind::SceneCommandHandler,
                "vector.scene",
            ),
        });

        let vector_ids: Vec<_> = registry
            .descriptors_by_domain("amigo.2d.vector")
            .map(|descriptor| descriptor.id.as_str())
            .collect();

        assert_eq!(registry.count_app_legacy(), 2);
        assert_eq!(registry.count_app_host(), 1);
        assert_eq!(vector_ids, vec!["vector.scene"]);
    }
}
