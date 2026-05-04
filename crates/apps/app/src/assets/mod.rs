//! App-specific asset helpers and adapters.
//! This module keeps application-facing asset wiring separate from the engine asset catalog crate.

use super::*;
use crate::runtime_context::RuntimeContext;
use crate::scene_runtime::current_loaded_scene_document_summary;

pub(super) fn process_pending_asset_loads(runtime: &Runtime) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let asset_catalog = ctx.required::<AssetCatalog>()?;
    let mod_catalog = ctx.required::<ModCatalog>()?;
    let dev_console_state = ctx.required::<DevConsoleState>()?;
    let sprite_scene_service = ctx.required::<SpriteSceneService>()?;
    let tilemap_scene_service = ctx.required::<TileMap2dSceneService>()?;

    for request in asset_catalog.drain_pending_loads() {
        let Some(manifest) = asset_catalog.manifest(&request.key) else {
            asset_catalog.mark_failed(
                request.key.clone(),
                "asset manifest missing for pending load request",
            );
            continue;
        };

        match resolve_asset_request_path(mod_catalog.as_ref(), &manifest.source, &request.key) {
            Ok(resolved_path) => match fs::metadata(&resolved_path) {
                Ok(metadata) if metadata.is_file() => {
                    let loaded_asset = amigo_assets::LoadedAsset {
                        key: request.key.clone(),
                        source: manifest.source.clone(),
                        resolved_path: resolved_path.clone(),
                        byte_len: metadata.len(),
                    };
                    asset_catalog.mark_loaded(loaded_asset.clone());
                    dev_console_state.write_line(format!(
                        "resolved asset `{}` to `{}` ({} bytes)",
                        request.key.as_str(),
                        resolved_path.display(),
                        metadata.len()
                    ));
                    if let Err(reason) = prepare_loaded_asset(
                        asset_catalog.as_ref(),
                        &loaded_asset,
                        dev_console_state.as_ref(),
                    ) {
                        asset_catalog.mark_failed(request.key.clone(), reason.clone());
                        dev_console_state.write_line(format!(
                            "asset prepare failed for `{}`: {reason}",
                            request.key.as_str()
                        ));
                    } else {
                        sync_sprite_sheet_metadata(
                            asset_catalog.as_ref(),
                            sprite_scene_service.as_ref(),
                            &loaded_asset.key,
                        );
                        sync_tile_ruleset_metadata(
                            asset_catalog.as_ref(),
                            tilemap_scene_service.as_ref(),
                            &loaded_asset.key,
                        );
                    }
                }
                Ok(_) => {
                    let reason = format!(
                        "resolved asset path `{}` is not a file",
                        resolved_path.display()
                    );
                    asset_catalog.mark_failed(request.key.clone(), reason.clone());
                    dev_console_state.write_line(format!(
                        "asset load failed for `{}`: {reason}",
                        request.key.as_str()
                    ));
                }
                Err(error) => {
                    let reason = format!(
                        "failed to access resolved asset path `{}`: {error}",
                        resolved_path.display()
                    );
                    asset_catalog.mark_failed(request.key.clone(), reason.clone());
                    dev_console_state.write_line(format!(
                        "asset load failed for `{}`: {reason}",
                        request.key.as_str()
                    ));
                }
            },
            Err(reason) => {
                asset_catalog.mark_failed(request.key.clone(), reason.clone());
                dev_console_state.write_line(format!(
                    "asset load failed for `{}`: {reason}",
                    request.key.as_str()
                ));
            }
        }
    }

    Ok(())
}

fn sync_sprite_sheet_metadata(
    asset_catalog: &AssetCatalog,
    sprite_scene_service: &SpriteSceneService,
    asset_key: &AssetKey,
) {
    let Some(prepared) = asset_catalog.prepared_asset(asset_key) else {
        return;
    };
    let Some(sheet) = amigo_2d_sprite::infer_sprite_sheet_from_prepared_asset(&prepared) else {
        return;
    };
    sprite_scene_service.sync_sheet_for_texture(asset_key, sheet);
}

fn sync_tile_ruleset_metadata(
    asset_catalog: &AssetCatalog,
    tilemap_scene_service: &TileMap2dSceneService,
    asset_key: &AssetKey,
) {
    let Some(prepared) = asset_catalog.prepared_asset(asset_key) else {
        return;
    };
    let Some(ruleset) = amigo_2d_tilemap::infer_tile_ruleset_from_prepared_asset(&prepared) else {
        return;
    };
    tilemap_scene_service.sync_ruleset_for_asset(asset_key, &ruleset);
}

pub(super) fn resolve_sprite_sheet_for_command(
    asset_catalog: &AssetCatalog,
    command: &Sprite2dSceneCommand,
) -> Option<SpriteSheet> {
    amigo_2d_sprite::resolve_sprite_sheet_for_command(asset_catalog, command)
}

pub(super) fn sync_hot_reload_watches(runtime: &Runtime) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let hot_reload = ctx.required::<HotReloadService>()?;
    let mod_catalog = ctx.required::<ModCatalog>()?;
    let asset_catalog = ctx.required::<AssetCatalog>()?;

    let scene_watch = current_loaded_scene_document_summary(runtime)?.and_then(|document| {
        mod_catalog
            .mod_by_id(&document.source_mod)
            .map(|discovered_mod| SceneDocumentWatch {
                source_mod: document.source_mod,
                scene_id: document.scene_id,
                path: discovered_mod.root_path.join(document.relative_path),
            })
    });
    hot_reload.sync_scene_document(scene_watch);

    let asset_watches = asset_catalog
        .manifests()
        .into_iter()
        .filter_map(|manifest| {
            resolve_asset_request_path(mod_catalog.as_ref(), &manifest.source, &manifest.key)
                .ok()
                .map(|path| AssetWatch {
                    asset_key: manifest.key.as_str().to_owned(),
                    path,
                })
        })
        .collect::<Vec<_>>();
    hot_reload.sync_assets(asset_watches);

    if let Some(file_watch_service) = ctx.optional::<FileWatchService>() {
        let watched_paths = hot_reload
            .watched_targets()
            .into_iter()
            .map(|watch| watch.path)
            .collect::<Vec<_>>();
        file_watch_service.sync_paths(&watched_paths)?;
    }

    Ok(())
}

pub(super) fn queue_hot_reload_changes(runtime: &Runtime) -> AmigoResult<usize> {
    let ctx = RuntimeContext::new(runtime);
    let hot_reload = ctx.required::<HotReloadService>()?;
    let scene_command_queue = ctx.required::<SceneCommandQueue>()?;
    let script_event_queue = ctx.required::<ScriptEventQueue>()?;
    let dev_console_state = ctx.required::<DevConsoleState>()?;
    let asset_catalog = ctx.required::<AssetCatalog>()?;
    let native_changes = ctx
        .optional::<FileWatchService>()
        .map(|file_watch_service| {
            let changed_paths = file_watch_service
                .drain_events()
                .into_iter()
                .map(|event| event.path)
                .collect::<Vec<_>>();
            hot_reload.changes_for_paths(&changed_paths)
        })
        .unwrap_or_default();
    let changes = if native_changes.is_empty() {
        hot_reload.poll_changes()
    } else {
        native_changes
    };
    for change in &changes {
        match &change.watch.kind {
            HotReloadWatchKind::SceneDocument {
                source_mod,
                scene_id,
            } => {
                dev_console_state.write_line(format!(
                    "detected scene document change for `{source_mod}:{scene_id}`"
                ));
                scene_command_queue.submit(SceneCommand::ReloadActiveScene);
            }
            HotReloadWatchKind::Asset { asset_key } => {
                dev_console_state.write_line(format!("detected asset change for `{asset_key}`"));
                crate::orchestration::request_asset_reload(
                    asset_catalog.as_ref(),
                    asset_key,
                    AssetLoadPriority::Immediate,
                    dev_console_state.as_ref(),
                );
                script_event_queue.publish(ScriptEvent::new(
                    "hot-reload.asset-changed",
                    vec![asset_key.clone()],
                ));
            }
        }
    }

    Ok(changes.len())
}

fn prepare_loaded_asset(
    asset_catalog: &AssetCatalog,
    loaded_asset: &amigo_assets::LoadedAsset,
    dev_console_state: &DevConsoleState,
) -> Result<(), String> {
    let contents = fs::read_to_string(&loaded_asset.resolved_path).map_err(|error| {
        format!(
            "failed to read loaded asset path `{}`: {error}",
            loaded_asset.resolved_path.display()
        )
    })?;
    let prepared = prepare_asset_from_contents(loaded_asset, &contents).map_err(|error| {
        format!(
            "failed to prepare asset `{}` from `{}`: {error}",
            loaded_asset.key.as_str(),
            loaded_asset.resolved_path.display()
        )
    })?;
    let kind = prepared.kind.as_str().to_owned();
    asset_catalog.mark_prepared(prepared);
    dev_console_state.write_line(format!(
        "prepared asset `{}` as `{kind}`",
        loaded_asset.key.as_str()
    ));

    Ok(())
}

pub(super) fn resolve_asset_request_path(
    mod_catalog: &ModCatalog,
    source: &AssetSourceKind,
    asset_key: &AssetKey,
) -> Result<PathBuf, String> {
    match source {
        AssetSourceKind::Mod(mod_id) => resolve_mod_asset_path(mod_catalog, mod_id, asset_key),
        AssetSourceKind::Engine => {
            let relative = safe_relative_asset_path(asset_key.as_str())?;
            resolve_existing_asset_path(PathBuf::from("assets").join(relative), asset_key.as_str())
        }
        AssetSourceKind::FileSystemRoot(root) => {
            let relative = safe_relative_asset_path(asset_key.as_str())?;
            resolve_existing_asset_path(PathBuf::from(root).join(relative), asset_key.as_str())
        }
        AssetSourceKind::Generated => Err(format!(
            "generated asset `{}` cannot be resolved from filesystem",
            asset_key.as_str()
        )),
    }
}

fn resolve_mod_asset_path(
    mod_catalog: &ModCatalog,
    mod_id: &str,
    asset_key: &AssetKey,
) -> Result<PathBuf, String> {
    let discovered_mod = mod_catalog.mod_by_id(mod_id).ok_or_else(|| {
        format!(
            "mod `{mod_id}` not found while resolving asset `{}`",
            asset_key.as_str()
        )
    })?;
    let Some(relative_key) = asset_key.as_str().strip_prefix(&format!("{mod_id}/")) else {
        return Err(format!(
            "asset key `{}` does not match mod source `{mod_id}`",
            asset_key.as_str()
        ));
    };
    let relative = safe_relative_asset_path(relative_key)?;
    if let Some(descriptor_path) =
        resolve_descriptor_first_asset_path(&discovered_mod.root_path, relative_key)
    {
        return Ok(descriptor_path);
    }

    resolve_existing_asset_path(discovered_mod.root_path.join(relative), asset_key.as_str())
}

fn resolve_descriptor_first_asset_path(mod_root: &Path, relative_key: &str) -> Option<PathBuf> {
    let normalized = relative_key.replace('\\', "/");
    let mut parts = normalized.split('/').collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let area = parts.remove(0);
    let id = parts.join("/");
    let suffixes = match area {
        "images" => &["image"][..],
        "sprites" => &["sprite", "atlas"],
        "tilesets" => &["tileset", "tile-ruleset"],
        "tilemaps" => &["tilemap"],
        "fonts" => &["font"],
        "audio" => &["audio"],
        "particles" => &["particle"],
        "materials" => &["material"],
        "ui" => &["ui"],
        _ => return None,
    };

    suffixes.iter().find_map(|suffix| {
        let candidate = mod_root
            .join("assets")
            .join(area)
            .join(format!("{id}.{suffix}.yml"));
        candidate.is_file().then_some(candidate)
    })
}

pub(super) fn resolve_existing_asset_path(
    base_path: PathBuf,
    asset_key: &str,
) -> Result<PathBuf, String> {
    if base_path.is_file() {
        return Ok(base_path);
    }

    for extension in ["yml", "yaml", "toml"] {
        let candidate = base_path.with_extension(extension);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "resolved asset path for `{asset_key}` does not exist as a file or known metadata candidate"
    ))
}

fn safe_relative_asset_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);

    if path.as_os_str().is_empty() {
        return Err("asset path must not be empty".to_owned());
    }

    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "asset path `{value}` must stay relative and inside its source root"
        ));
    }

    Ok(path)
}
