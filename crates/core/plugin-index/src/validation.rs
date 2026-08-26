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

    for duplicate in index.duplicate_ids() {
        errors.push(PluginIndexValidationError::DuplicatePluginId(
            duplicate.0.clone(),
        ));
    }

    for manifest in index.manifests() {
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
