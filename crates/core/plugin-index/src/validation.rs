use std::collections::HashSet;

use amigo_plugin_api::{validate_plugin_manifest, PluginId, PluginManifestValidationError};

use crate::index::PluginIndex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginIndexValidationError {
    InvalidManifest {
        plugin: PluginId,
        errors: Vec<PluginManifestValidationError>,
    },
    DuplicatePluginId(String),
    MissingRequiredCapability {
        plugin: String,
        capability: String,
    },
    MissingRequiredSlot {
        plugin: String,
        slot: String,
    },
    MissingReplacementPlugin {
        plugin: String,
        replacement: String,
    },
    MissingContributionProducer {
        plugin: String,
        contribution: String,
    },
}

pub type PluginIndexValidationResult = Result<(), Vec<PluginIndexValidationError>>;

pub fn validate_plugin_index(index: &PluginIndex) -> PluginIndexValidationResult {
    let mut errors = Vec::new();

    for duplicate in index.duplicate_ids() {
        errors.push(PluginIndexValidationError::DuplicatePluginId(
            duplicate.0.clone(),
        ));
    }

    let plugin_ids = index
        .manifests()
        .map(|manifest| manifest.id.0.clone())
        .collect::<HashSet<_>>();
    let provided_capabilities = index
        .manifests()
        .flat_map(|manifest| manifest.capabilities.provides.iter())
        .map(|capability| (capability.id.0.clone(), capability.version))
        .collect::<HashSet<_>>();
    let implemented_slots = index
        .manifests()
        .flat_map(|manifest| manifest.slots.implements.iter())
        .map(|slot| slot.0.clone())
        .collect::<HashSet<_>>();
    let emitted_contributions = index
        .manifests()
        .flat_map(|manifest| manifest.contributions.emits.iter())
        .map(|contribution| {
            (
                contribution.domain.0.clone(),
                contribution.contribution_type.clone(),
            )
        })
        .collect::<HashSet<_>>();

    for manifest in index.manifests() {
        if let Err(manifest_errors) = validate_plugin_manifest(manifest) {
            errors.push(PluginIndexValidationError::InvalidManifest {
                plugin: manifest.id.clone(),
                errors: manifest_errors,
            });
        }

        for capability in &manifest.capabilities.requires {
            if !provided_capabilities.contains(&(capability.id.0.clone(), capability.version)) {
                errors.push(PluginIndexValidationError::MissingRequiredCapability {
                    plugin: manifest.id.0.clone(),
                    capability: format!("{}@{}", capability.id.0, capability.version),
                });
            }
        }

        for slot in &manifest.slots.requires {
            if !implemented_slots.contains(&slot.0) {
                errors.push(PluginIndexValidationError::MissingRequiredSlot {
                    plugin: manifest.id.0.clone(),
                    slot: slot.0.clone(),
                });
            }
        }

        for replacement in &manifest.slots.replaces {
            if !plugin_ids.contains(&replacement.0) {
                errors.push(PluginIndexValidationError::MissingReplacementPlugin {
                    plugin: manifest.id.0.clone(),
                    replacement: replacement.0.clone(),
                });
            }
        }

        for contribution in &manifest.contributions.consumes {
            let key = (
                contribution.domain.0.clone(),
                contribution.contribution_type.clone(),
            );
            if !emitted_contributions.contains(&key) {
                errors.push(PluginIndexValidationError::MissingContributionProducer {
                    plugin: manifest.id.0.clone(),
                    contribution: format!(
                        "{}::{}",
                        contribution.domain.0, contribution.contribution_type
                    ),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
