use crate::ids::{DomainId, PluginId};
use crate::status::ContributionStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContributionPolicy {
    ExplicitOnly,
    EnabledByDefault,
    DisabledByDefault,
    DerivedAtHydration,
    Forbidden,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionTrace {
    pub source_plugin: PluginId,
    pub domain: DomainId,
    pub adapter: String,
    pub policy: ContributionPolicy,
    pub status: ContributionStatus,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionContract {
    pub domain: DomainId,
    pub contribution_type: String,
    pub policy: ContributionPolicy,
}

impl ContributionContract {
    pub fn is_empty(&self) -> bool {
        self.domain.0.trim().is_empty() || self.contribution_type.trim().is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContributionManifest {
    pub emits: Vec<ContributionContract>,
    pub consumes: Vec<ContributionContract>,
}

pub trait DomainContribution {
    fn domain(&self) -> DomainId;
    fn status(&self) -> ContributionStatus;
    fn trace(&self) -> Option<&ContributionTrace>;
}
