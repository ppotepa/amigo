use std::collections::HashSet;

use crate::ids::{DiagnosticChannelId, PluginId, SlotId, TargetId};
use crate::manifest::PluginManifest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginManifestValidationError {
    EmptyPluginId,
    EmptyFamilyId,
    EmptyCapabilityId,
    ZeroCapabilityVersion,
    EmptySlotId,
    EmptyTargetId,
    EmptyDiagnosticChannelId,
    EmptyDiagnosticOwner,
    MissingPipelineDocs,
    MissingWaterfallTest,
    DuplicateProvidedCapability(String),
    DuplicateRequiredCapability(String),
    DuplicateImplementedSlot(String),
    DuplicateRequiredSlot(String),
    DuplicateTargetRead(String),
    DuplicateTargetWrite(String),
    DuplicateTargetContribution(String),
    DuplicateDiagnosticChannel(String),
    EmptyContributionDomain,
    EmptyContributionType,
}

pub type PluginManifestValidationResult = Result<(), Vec<PluginManifestValidationError>>;

pub fn validate_plugin_manifest(manifest: &PluginManifest) -> PluginManifestValidationResult {
    let mut errors = Vec::new();

    if manifest.id.0.trim().is_empty() {
        errors.push(PluginManifestValidationError::EmptyPluginId);
    }

    if manifest.family.0.trim().is_empty() {
        errors.push(PluginManifestValidationError::EmptyFamilyId);
    }

    validate_capabilities(manifest, &mut errors);
    validate_slots(manifest, &mut errors);
    validate_targets(manifest, &mut errors);
    validate_contributions(manifest, &mut errors);
    validate_diagnostics(manifest, &mut errors);
    validate_docs_and_tests(manifest, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_capabilities(
    manifest: &PluginManifest,
    errors: &mut Vec<PluginManifestValidationError>,
) {
    let mut provides = HashSet::new();
    let mut requires = HashSet::new();

    for capability in &manifest.capabilities.provides {
        if capability.id.0.trim().is_empty() {
            errors.push(PluginManifestValidationError::EmptyCapabilityId);
        }
        if capability.version == 0 {
            errors.push(PluginManifestValidationError::ZeroCapabilityVersion);
        }
        let key = format!("{}@{}", capability.id.0, capability.version);
        if !provides.insert(key.clone()) {
            errors.push(PluginManifestValidationError::DuplicateProvidedCapability(
                key,
            ));
        }
    }

    for capability in &manifest.capabilities.requires {
        if capability.id.0.trim().is_empty() {
            errors.push(PluginManifestValidationError::EmptyCapabilityId);
        }
        if capability.version == 0 {
            errors.push(PluginManifestValidationError::ZeroCapabilityVersion);
        }
        let key = format!("{}@{}", capability.id.0, capability.version);
        if !requires.insert(key.clone()) {
            errors.push(PluginManifestValidationError::DuplicateRequiredCapability(
                key,
            ));
        }
    }
}

fn validate_slots(manifest: &PluginManifest, errors: &mut Vec<PluginManifestValidationError>) {
    validate_slot_list(
        &manifest.slots.implements,
        errors,
        PluginManifestValidationError::DuplicateImplementedSlot,
    );

    validate_slot_list(
        &manifest.slots.requires,
        errors,
        PluginManifestValidationError::DuplicateRequiredSlot,
    );
}

fn validate_slot_list(
    slots: &[SlotId],
    errors: &mut Vec<PluginManifestValidationError>,
    duplicate: fn(String) -> PluginManifestValidationError,
) {
    let mut seen = HashSet::new();

    for slot in slots {
        if slot.0.trim().is_empty() {
            errors.push(PluginManifestValidationError::EmptySlotId);
        }

        if !seen.insert(slot.0.clone()) {
            errors.push(duplicate(slot.0.clone()));
        }
    }
}

fn validate_targets(manifest: &PluginManifest, errors: &mut Vec<PluginManifestValidationError>) {
    validate_target_list(
        &manifest.targets.reads,
        errors,
        PluginManifestValidationError::DuplicateTargetRead,
    );

    validate_target_list(
        &manifest.targets.writes,
        errors,
        PluginManifestValidationError::DuplicateTargetWrite,
    );

    validate_target_list(
        &manifest.targets.contributes,
        errors,
        PluginManifestValidationError::DuplicateTargetContribution,
    );
}

fn validate_target_list(
    targets: &[TargetId],
    errors: &mut Vec<PluginManifestValidationError>,
    duplicate: fn(String) -> PluginManifestValidationError,
) {
    let mut seen = HashSet::new();

    for target in targets {
        if target.0.trim().is_empty() {
            errors.push(PluginManifestValidationError::EmptyTargetId);
        }

        if !seen.insert(target.0.clone()) {
            errors.push(duplicate(target.0.clone()));
        }
    }
}

fn validate_contributions(
    manifest: &PluginManifest,
    errors: &mut Vec<PluginManifestValidationError>,
) {
    for contribution in manifest
        .contributions
        .emits
        .iter()
        .chain(manifest.contributions.consumes.iter())
    {
        if contribution.domain.0.trim().is_empty() {
            errors.push(PluginManifestValidationError::EmptyContributionDomain);
        }

        if contribution.contribution_type.trim().is_empty() {
            errors.push(PluginManifestValidationError::EmptyContributionType);
        }
    }
}

fn validate_diagnostics(
    manifest: &PluginManifest,
    errors: &mut Vec<PluginManifestValidationError>,
) {
    let mut seen = HashSet::new();

    for diagnostic in &manifest.diagnostics.channels {
        validate_diagnostic_channel(&diagnostic.id, errors);
        validate_diagnostic_owner(&diagnostic.owner, errors);

        if !seen.insert(diagnostic.id.0.clone()) {
            errors.push(PluginManifestValidationError::DuplicateDiagnosticChannel(
                diagnostic.id.0.clone(),
            ));
        }
    }
}

fn validate_diagnostic_channel(
    channel: &DiagnosticChannelId,
    errors: &mut Vec<PluginManifestValidationError>,
) {
    if channel.0.trim().is_empty() {
        errors.push(PluginManifestValidationError::EmptyDiagnosticChannelId);
    }
}

fn validate_diagnostic_owner(owner: &PluginId, errors: &mut Vec<PluginManifestValidationError>) {
    if owner.0.trim().is_empty() {
        errors.push(PluginManifestValidationError::EmptyDiagnosticOwner);
    }
}

fn validate_docs_and_tests(
    manifest: &PluginManifest,
    errors: &mut Vec<PluginManifestValidationError>,
) {
    if manifest
        .docs
        .pipeline
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        errors.push(PluginManifestValidationError::MissingPipelineDocs);
    }

    if manifest
        .tests
        .waterfall
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        errors.push(PluginManifestValidationError::MissingWaterfallTest);
    }
}
