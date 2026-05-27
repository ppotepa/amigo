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
}

pub type PluginIndexValidationResult = Result<(), Vec<PluginIndexValidationError>>;

pub fn validate_plugin_index(index: &PluginIndex) -> PluginIndexValidationResult {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();

    for manifest in index.manifests() {
        if !seen.insert(manifest.id.0.clone()) {
            errors.push(PluginIndexValidationError::DuplicatePluginId(
                manifest.id.0.clone(),
            ));
        }

        if let Err(manifest_errors) = validate_plugin_manifest(manifest) {
            errors.push(PluginIndexValidationError::InvalidManifest {
                plugin: manifest.id.clone(),
                errors: manifest_errors,
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
