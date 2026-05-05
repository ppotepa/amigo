use std::path::Path;

use tauri::{AppHandle, State};

use crate::asset_registry::dto::{
    AssetRegistryDto, CreateAssetDescriptorRequestDto, CreateSpritesheetRulesetRequestDto,
    ManagedAssetDto,
};
use crate::events::bus;
use crate::session::EditorSessionRegistry;

pub fn get_asset_registry(
    session_id: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<AssetRegistryDto, String> {
    let session = sessions.get_session(&session_id)?;
    let registry = crate::asset_registry::scanner::scan_asset_registry(
        &session.session_id,
        &session.mod_id,
        Path::new(&session.root_path),
    )?;
    Ok(crate::asset_registry::graph::build_asset_graph(registry))
}

pub fn create_asset_descriptor(
    app: AppHandle,
    session_id: String,
    request: CreateAssetDescriptorRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ManagedAssetDto, String> {
    let session = sessions.get_session(&session_id)?;
    let asset = crate::asset_registry::scanner::create_asset_descriptor(
        &session.mod_id,
        Path::new(&session.root_path),
        request,
    )?;
    let _ = bus::emit_asset_descriptor_changed(
        &app,
        session.mod_id.clone(),
        asset.asset_key.clone(),
        asset.descriptor_relative_path.clone(),
        "created",
    );
    let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        Some(session.mod_id.clone()),
        None,
        None,
        "asset",
        "asset descriptor created",
    );
    Ok(asset)
}

pub fn create_spritesheet_ruleset(
    app: AppHandle,
    session_id: String,
    request: CreateSpritesheetRulesetRequestDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<ManagedAssetDto, String> {
    let session = sessions.get_session(&session_id)?;
    let asset = crate::asset_registry::scanner::create_spritesheet_ruleset(
        &session.mod_id,
        Path::new(&session.root_path),
        request,
    )?;
    let _ = bus::emit_asset_descriptor_changed(
        &app,
        session.mod_id.clone(),
        asset.asset_key.clone(),
        asset.descriptor_relative_path.clone(),
        "created",
    );
    let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        Some(session.mod_id.clone()),
        None,
        None,
        "asset",
        "spritesheet ruleset created",
    );
    Ok(asset)
}
