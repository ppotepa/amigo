use std::path::Path;

use tauri::{AppHandle, State};

use crate::events::bus;
use crate::session::EditorSessionRegistry;
use crate::sheet::dto::{SheetResourceDto, TileRulesetResourceDto, TilemapResourceDto};

use super::project_files::asset_key_from_descriptor_relative_path;

pub fn load_sheet_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<SheetResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::sheet::loader::load_sheet_resource(Path::new(&session.root_path), &resource_uri)
}

pub fn load_tilemap_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<TilemapResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::sheet::loader::load_tilemap_resource(Path::new(&session.root_path), &resource_uri)
}

pub fn load_tile_ruleset_resource(
    session_id: String,
    resource_uri: String,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<TileRulesetResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    crate::sheet::loader::load_tile_ruleset_resource(Path::new(&session.root_path), &resource_uri)
}

pub fn save_tilemap_resource(
    app: AppHandle,
    session_id: String,
    resource_uri: String,
    tilemap: TilemapResourceDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<TilemapResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    let saved = crate::sheet::loader::save_tilemap_resource(
        Path::new(&session.root_path),
        &resource_uri,
        tilemap,
    )?;
    let asset_key = asset_key_from_descriptor_relative_path(&session.mod_id, &saved.relative_path)
        .unwrap_or_else(|| saved.relative_path.clone());
    let _ = bus::emit_asset_descriptor_changed(
        &app,
        session.mod_id.clone(),
        asset_key,
        saved.relative_path.clone(),
        "saved",
    );
    let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        Some(session.mod_id.clone()),
        None,
        None,
        "tilemap",
        "tilemap resource saved",
    );
    Ok(saved)
}

pub fn save_sheet_resource(
    app: AppHandle,
    session_id: String,
    resource_uri: String,
    sheet: SheetResourceDto,
    sessions: State<'_, EditorSessionRegistry>,
) -> Result<SheetResourceDto, String> {
    let session = sessions.get_session(&session_id)?;
    let saved =
        crate::sheet::loader::save_sheet_resource(Path::new(&session.root_path), &resource_uri, sheet)?;
    let asset_key = asset_key_from_descriptor_relative_path(&session.mod_id, &saved.relative_path)
        .unwrap_or_else(|| saved.relative_path.clone());
    let _ = bus::emit_asset_descriptor_changed(
        &app,
        session.mod_id.clone(),
        asset_key,
        saved.relative_path.clone(),
        "saved",
    );
    let _ = bus::emit_asset_registry_changed(&app, session.mod_id.clone());
    let _ = bus::emit_cache_invalidated(
        &app,
        None,
        Some(session.mod_id.clone()),
        None,
        None,
        "sheet",
        "sheet resource saved",
    );
    Ok(saved)
}
