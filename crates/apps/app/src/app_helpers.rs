use super::*;

pub(crate) fn relative_path_within_root(
    root_path: &Path,
    absolute_path: &Path,
) -> AmigoResult<PathBuf> {
    let relative_path = absolute_path.strip_prefix(root_path).map_err(|_| {
        AmigoError::Message(format!(
            "script path `{}` must stay within mod root `{}`",
            absolute_path.display(),
            root_path.display()
        ))
    })?;

    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(AmigoError::Message(format!(
            "script path `{}` resolved to an invalid relative path `{}`",
            absolute_path.display(),
            relative_path.display()
        )));
    }

    Ok(relative_path.to_path_buf())
}

pub(crate) fn validate_script_path(
    script_runtime: &ScriptRuntimeService,
    script_path: &Path,
    owner_label: &str,
) -> AmigoResult<()> {
    let extension = script_path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "{owner_label} `{}` has no file extension",
                script_path.display()
            ))
        })?;

    if !script_runtime.supports_extension(extension) {
        return Err(AmigoError::Message(format!(
            "{owner_label} `{}` is not supported by `{}`",
            script_path.display(),
            script_runtime.backend_name()
        )));
    }

    Ok(())
}

pub(crate) fn parse_scene_vec2(width: &str, height: &str, label: &str) -> Result<Vec2, String> {
    let width = width
        .parse::<f32>()
        .map_err(|error| format!("failed to parse {label} width `{width}` as f32: {error}"))?;
    let height = height
        .parse::<f32>()
        .map_err(|error| format!("failed to parse {label} height `{height}` as f32: {error}"))?;

    Ok(Vec2::new(width, height))
}

pub(crate) fn register_mod_asset_reference(
    asset_catalog: &AssetCatalog,
    source_mod: &str,
    asset_key: &AssetKey,
    domain_scope: &str,
    domain_tag: &str,
) {
    asset_catalog.register_manifest(AssetManifest {
        key: asset_key.clone(),
        source: AssetSourceKind::Mod(source_mod.to_owned()),
        tags: vec![
            "phase3".to_owned(),
            domain_scope.to_owned(),
            domain_tag.to_owned(),
        ],
    });
    asset_catalog.request_load(AssetLoadRequest::new(
        asset_key.clone(),
        AssetLoadPriority::Interactive,
    ));
}

pub(crate) fn register_audio_clip_reference(
    asset_catalog: &AssetCatalog,
    audio_scene_service: &AudioSceneService,
    asset_key: &AssetKey,
    mode: AudioPlaybackMode,
) {
    let source_mod = asset_key
        .as_str()
        .split('/')
        .next()
        .unwrap_or_default()
        .to_owned();
    if source_mod.is_empty() {
        return;
    }

    register_mod_asset_reference(asset_catalog, &source_mod, asset_key, "audio", "generated");
    audio_scene_service.register_clip(AudioClip {
        key: AudioClipKey::new(asset_key.as_str().to_owned()),
        mode,
    });
}

pub(crate) fn resolve_mod_audio_asset_key(
    launch_selection: &LaunchSelection,
    clip_name: &str,
) -> AssetKey {
    if clip_name.contains('/') {
        AssetKey::new(clip_name.to_owned())
    } else {
        AssetKey::new(format!(
            "{}/audio/{}",
            launch_selection.selected_mod(),
            clip_name
        ))
    }
}

pub(crate) fn format_script_command(command: &ScriptCommand) -> String {
    if command.arguments.is_empty() {
        return format!("{}.{}", command.namespace, command.name);
    }

    format!(
        "{}.{}({})",
        command.namespace,
        command.name,
        command.arguments.join(", ")
    )
}

pub(crate) fn format_audio_command(command: &AudioCommand) -> String {
    match command {
        AudioCommand::PlayOnce { clip } => format!("audio.play({})", clip.as_str()),
        AudioCommand::StartSource { source, clip } => {
            format!("audio.start({}, {})", source.as_str(), clip.as_str())
        }
        AudioCommand::StopSource { source } => format!("audio.stop({})", source.as_str()),
        AudioCommand::SetParam {
            source,
            param,
            value,
        } => format!("audio.set_param({}, {}, {})", source.as_str(), param, value),
        AudioCommand::SetVolume { bus, value } => {
            format!("audio.set_volume({}, {})", bus, value)
        }
        AudioCommand::SetMasterVolume { value } => format!("audio.set_master_volume({value})"),
    }
}

pub(crate) fn format_script_event(event: &ScriptEvent) -> String {
    if event.payload.is_empty() {
        return event.topic.clone();
    }

    format!("{}({})", event.topic, event.payload.join(", "))
}

pub(crate) fn display_string_list(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_owned();
    }

    values.join(", ")
}

pub(crate) fn display_executed_scripts(scripts: &[ExecutedScript]) -> String {
    if scripts.is_empty() {
        return "none".to_owned();
    }

    scripts
        .iter()
        .map(|script| script.source_name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}
