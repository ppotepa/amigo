use std::collections::{BTreeMap, BTreeSet, VecDeque};

use amigo_plugin_api::PluginId;

use crate::PluginIndex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCompositionPlan {
    pub ordered_plugins: Vec<PluginId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginCompositionPlanError {
    MissingCapabilityProvider { plugin: String, capability: String, version: u32 },
    AmbiguousCapabilityProvider { plugin: String, capability: String, version: u32, providers: Vec<String> },
    MissingSlotProvider { plugin: String, slot: String },
    AmbiguousSlotProvider { plugin: String, slot: String, providers: Vec<String> },
    DependencyCycle { plugins: Vec<String> },
}

pub fn plan_plugin_composition(index: &PluginIndex) -> Result<PluginCompositionPlan, PluginCompositionPlanError> {
    let manifests = index.manifests().collect::<Vec<_>>();
    let mut capability_providers: BTreeMap<(String, u32), Vec<String>> = BTreeMap::new();
    let mut slot_providers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut contribution_emitters: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for manifest in &manifests {
        for capability in &manifest.capabilities.provides {
            capability_providers
                .entry((capability.id.0.clone(), capability.version))
                .or_default()
                .push(manifest.id.0.clone());
        }
        for slot in &manifest.slots.implements {
            slot_providers.entry(slot.0.clone()).or_default().push(manifest.id.0.clone());
        }
        for contribution in &manifest.contributions.emits {
            contribution_emitters
                .entry((contribution.domain.0.clone(), contribution.contribution_type.clone()))
                .or_default()
                .push(manifest.id.0.clone());
        }
    }

    let mut dependencies: BTreeMap<String, BTreeSet<String>> = manifests
        .iter()
        .map(|manifest| (manifest.id.0.clone(), BTreeSet::new()))
        .collect();

    for manifest in &manifests {
        let deps = dependencies.get_mut(&manifest.id.0).expect("plugin dependency bucket exists");
        for requirement in &manifest.capabilities.requires {
            let key = (requirement.id.0.clone(), requirement.version);
            let providers = capability_providers.get(&key).cloned().unwrap_or_default();
            match providers.as_slice() {
                [] => return Err(PluginCompositionPlanError::MissingCapabilityProvider {
                    plugin: manifest.id.0.clone(), capability: requirement.id.0.clone(), version: requirement.version,
                }),
                [provider] => { if provider != &manifest.id.0 { deps.insert(provider.clone()); } }
                _ => return Err(PluginCompositionPlanError::AmbiguousCapabilityProvider {
                    plugin: manifest.id.0.clone(), capability: requirement.id.0.clone(), version: requirement.version, providers,
                }),
            }
        }
        for slot in &manifest.slots.requires {
            let providers = slot_providers.get(&slot.0).cloned().unwrap_or_default();
            match providers.as_slice() {
                [] => return Err(PluginCompositionPlanError::MissingSlotProvider {
                    plugin: manifest.id.0.clone(), slot: slot.0.clone(),
                }),
                [provider] => { if provider != &manifest.id.0 { deps.insert(provider.clone()); } }
                _ => return Err(PluginCompositionPlanError::AmbiguousSlotProvider {
                    plugin: manifest.id.0.clone(), slot: slot.0.clone(), providers,
                }),
            }
        }
        for contribution in &manifest.contributions.consumes {
            if let Some(emitters) = contribution_emitters.get(&(
                contribution.domain.0.clone(), contribution.contribution_type.clone(),
            )) {
                for emitter in emitters {
                    if emitter != &manifest.id.0 { deps.insert(emitter.clone()); }
                }
            }
        }
    }

    let mut indegree: BTreeMap<String, usize> = dependencies
        .iter()
        .map(|(plugin, deps)| (plugin.clone(), deps.len()))
        .collect();
    let mut reverse: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (plugin, deps) in &dependencies {
        for dependency in deps {
            reverse.entry(dependency.clone()).or_default().insert(plugin.clone());
        }
    }

    let mut ready = VecDeque::from(
        indegree.iter().filter_map(|(plugin, degree)| (*degree == 0).then_some(plugin.clone())).collect::<Vec<_>>()
    );
    let mut ordered = Vec::with_capacity(indegree.len());
    while let Some(plugin) = ready.pop_front() {
        ordered.push(plugin.clone());
        if let Some(consumers) = reverse.get(&plugin) {
            for consumer in consumers {
                let degree = indegree.get_mut(consumer).expect("consumer indegree exists");
                *degree -= 1;
                if *degree == 0 { ready.push_back(consumer.clone()); }
            }
        }
    }

    if ordered.len() != indegree.len() {
        let plugins = indegree
            .into_iter()
            .filter_map(|(plugin, degree)| (degree > 0).then_some(plugin))
            .collect();
        return Err(PluginCompositionPlanError::DependencyCycle { plugins });
    }

    Ok(PluginCompositionPlan {
        ordered_plugins: ordered.into_iter().map(PluginId).collect(),
    })
}

#[cfg(test)]
mod tests {
    use amigo_plugin_api::{CapabilityRef, PluginKind, PluginManifest, RenderParticipation};
    use super::*;

    fn manifest(id: &str) -> PluginManifest {
        PluginManifest::new(id, "test", PluginKind::SemanticSource, false, RenderParticipation::None)
    }

    #[test]
    fn plans_provider_before_consumer() {
        let mut provider = manifest("provider");
        provider.capabilities.provides.push(CapabilityRef::new("feature", 1));
        let mut consumer = manifest("consumer");
        consumer.capabilities.requires.push(CapabilityRef::new("feature", 1));
        let index = PluginIndex::from_manifests([consumer, provider]);
        let plan = plan_plugin_composition(&index).expect("composition should plan");
        assert_eq!(plan.ordered_plugins.iter().map(|id| id.0.as_str()).collect::<Vec<_>>(), vec!["provider", "consumer"]);
    }

    #[test]
    fn rejects_dependency_cycle() {
        let mut a = manifest("a");
        a.capabilities.provides.push(CapabilityRef::new("a", 1));
        a.capabilities.requires.push(CapabilityRef::new("b", 1));
        let mut b = manifest("b");
        b.capabilities.provides.push(CapabilityRef::new("b", 1));
        b.capabilities.requires.push(CapabilityRef::new("a", 1));
        let index = PluginIndex::from_manifests([a, b]);
        assert!(matches!(plan_plugin_composition(&index), Err(PluginCompositionPlanError::DependencyCycle { .. })));
    }
}
