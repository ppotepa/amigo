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

pub trait DomainContribution {
    fn domain(&self) -> DomainId;
    fn status(&self) -> ContributionStatus;
    fn trace(&self) -> Option<&ContributionTrace>;
}
