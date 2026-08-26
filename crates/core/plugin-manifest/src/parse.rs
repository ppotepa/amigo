use amigo_plugin_api::{
    CapabilityRef, ContributionContract, ContributionPolicy, DiagnosticChannelId,
    DiagnosticChannelRef, DomainId, PluginId, PluginKind, PluginManifest, RenderParticipation,
    SlotId, TargetId,
};

use crate::error::PluginManifestParseError;
use crate::raw::{RawContribution, RawPluginManifest};

pub fn parse_plugin_manifest_str(input: &str) -> Result<PluginManifest, PluginManifestParseError> {
    let raw: RawPluginManifest = toml::from_str(input)?;

    let kind = parse_kind(&raw.kind)?;
    let render_participation =
        parse_render_participation(raw.render_participation.as_deref().unwrap_or("none"))?;

    let mut manifest = PluginManifest::new(
        raw.id.clone(),
        raw.family.clone(),
        kind,
        raw.renderable,
        render_participation,
    );

    for item in raw.capabilities.provides {
        manifest.capabilities.provides.push(parse_capability(&item));
    }
    for item in raw.capabilities.requires {
        manifest.capabilities.requires.push(parse_capability(&item));
    }
    for item in raw.slots.implements {
        manifest.slots.implements.push(SlotId(item));
    }
    for item in raw.slots.requires {
        manifest.slots.requires.push(SlotId(item));
    }
    for item in raw.slots.replaces {
        manifest.slots.replaces.push(PluginId(item));
    }
    for item in raw.targets.reads {
        manifest.targets.reads.push(TargetId(item));
    }
    for item in raw.targets.writes {
        manifest.targets.writes.push(TargetId(item));
    }
    for item in raw.targets.contributes {
        manifest.targets.contributes.push(TargetId(item));
    }
    for contribution in raw.contributions.emits {
        manifest.contributions.emits.push(parse_contribution(contribution)?);
    }
    for contribution in raw.contributions.consumes {
        manifest.contributions.consumes.push(parse_contribution(contribution)?);
    }
    for channel in raw.diagnostics.channels {
        manifest.diagnostics.channels.push(DiagnosticChannelRef {
            id: DiagnosticChannelId(channel),
            owner: manifest.id.clone(),
        });
    }

    manifest.docs.pipeline = raw.docs.pipeline;
    manifest.docs.contributions = raw.docs.contributions;
    manifest.docs.diagnostics = raw.docs.diagnostics;
    manifest.tests.hydration = raw.tests.hydration;
    manifest.tests.participation = raw.tests.participation;
    manifest.tests.candidate = raw.tests.candidate;
    manifest.tests.waterfall = raw.tests.waterfall;
    manifest.tests.diagnostics = raw.tests.diagnostics;
    Ok(manifest)
}

fn parse_kind(value: &str) -> Result<PluginKind, PluginManifestParseError> {
    match value {
        "renderable-source" => Ok(PluginKind::RenderableSource),
        "semantic-source" => Ok(PluginKind::SemanticSource),
        "target-consumer" => Ok(PluginKind::TargetConsumer),
        "source-and-consumer" => Ok(PluginKind::SourceAndConsumer),
        "bundle" => Ok(PluginKind::Bundle),
        "adapter" => Ok(PluginKind::Adapter),
        "tooling" => Ok(PluginKind::Tooling),
        "noop" => Ok(PluginKind::Noop),
        other => Err(PluginManifestParseError::UnknownPluginKind(other.to_string())),
    }
}

fn parse_render_participation(value: &str) -> Result<RenderParticipation, PluginManifestParseError> {
    match value {
        "none" => Ok(RenderParticipation::None),
        "source-renderer" => Ok(RenderParticipation::SourceRenderer),
        "target-writer" => Ok(RenderParticipation::TargetWriter),
        "target-consumer" => Ok(RenderParticipation::TargetConsumer),
        "render-backend" => Ok(RenderParticipation::RenderBackend),
        other => Err(PluginManifestParseError::UnknownRenderParticipation(other.to_string())),
    }
}

fn parse_capability(value: &str) -> CapabilityRef {
    let mut parts = value.split('@');
    let id = parts.next().unwrap_or_default().to_string();
    let version = parts
        .next()
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(1);
    CapabilityRef::new(id, version)
}

fn parse_contribution(raw: RawContribution) -> Result<ContributionContract, PluginManifestParseError> {
    Ok(ContributionContract {
        domain: DomainId(raw.domain),
        contribution_type: raw.contribution_type,
        policy: parse_policy(&raw.policy)?,
    })
}

fn parse_policy(value: &str) -> Result<ContributionPolicy, PluginManifestParseError> {
    match value {
        "EnabledByDefault" => Ok(ContributionPolicy::EnabledByDefault),
        "DisabledByDefault" => Ok(ContributionPolicy::DisabledByDefault),
        "DerivedAtHydration" => Ok(ContributionPolicy::DerivedAtHydration),
        "Forbidden" => Ok(ContributionPolicy::Forbidden),
        "ExplicitOnly" => Ok(ContributionPolicy::ExplicitOnly),
        other => Err(PluginManifestParseError::UnknownContributionPolicy(other.to_owned())),
    }
}
