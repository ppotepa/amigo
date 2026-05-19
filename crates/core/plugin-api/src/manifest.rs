use crate::capability::CapabilityRef;
use crate::diagnostics::DiagnosticChannelRef;
use crate::ids::{FamilyId, PluginId, SlotId};
use crate::kinds::{PluginKind, RenderParticipation};
use crate::target::TargetRef;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: PluginId,
    pub family: FamilyId,
    pub kind: PluginKind,
    pub render_participation: RenderParticipation,
    pub provides: Vec<CapabilityRef>,
    pub requires: Vec<CapabilityRef>,
    pub implements_slots: Vec<SlotId>,
    pub reads_writes_targets: Vec<TargetRef>,
    pub diagnostics: Vec<DiagnosticChannelRef>,
}
