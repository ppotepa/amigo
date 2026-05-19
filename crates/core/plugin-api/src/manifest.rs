use crate::capability::CapabilityManifest;
use crate::contribution::ContributionManifest;
use crate::diagnostics::DiagnosticManifest;
use crate::ids::{FamilyId, PluginId};
use crate::kinds::{PluginKind, RenderParticipation};
use crate::slot::SlotManifest;
use crate::target::TargetManifest;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginDocsManifest {
    pub pipeline: Option<String>,
    pub contributions: Option<String>,
    pub diagnostics: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginTestsManifest {
    pub hydration: Option<String>,
    pub participation: Option<String>,
    pub candidate: Option<String>,
    pub waterfall: Option<String>,
    pub diagnostics: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: PluginId,
    pub family: FamilyId,
    pub kind: PluginKind,
    pub renderable: bool,
    pub render_participation: RenderParticipation,
    pub capabilities: CapabilityManifest,
    pub slots: SlotManifest,
    pub targets: TargetManifest,
    pub contributions: ContributionManifest,
    pub diagnostics: DiagnosticManifest,
    pub docs: PluginDocsManifest,
    pub tests: PluginTestsManifest,
}

impl PluginManifest {
    pub fn new(
        id: impl Into<String>,
        family: impl Into<String>,
        kind: PluginKind,
        renderable: bool,
        render_participation: RenderParticipation,
    ) -> Self {
        Self {
            id: PluginId(id.into()),
            family: FamilyId(family.into()),
            kind,
            renderable,
            render_participation,
            capabilities: CapabilityManifest::default(),
            slots: SlotManifest::default(),
            targets: TargetManifest::default(),
            contributions: ContributionManifest::default(),
            diagnostics: DiagnosticManifest::default(),
            docs: PluginDocsManifest::default(),
            tests: PluginTestsManifest::default(),
        }
    }
}
