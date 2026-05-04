use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::events::envelope::WindowEventEnvelope;
use crate::events::names;
use crate::events::payloads::{
    AssetDescriptorChangedPayload, AssetRegistryChangedPayload, CacheInvalidatedPayload,
    FontSettingsChangedPayload, SessionClosedPayload, ThemeSettingsChangedPayload,
};

pub fn emit_window_event<T>(
    app: &AppHandle,
    event_name: &'static str,
    source_window: Option<String>,
    session_id: Option<String>,
    payload: T,
) -> Result<(), String>
where
    T: Clone + Serialize,
{
    let envelope = WindowEventEnvelope {
        event_id: new_event_id(),
        event_type: event_name.to_owned(),
        source_window,
        session_id,
        timestamp_ms: now_ms(),
        schema_version: 1,
        payload,
    };

    app.emit(event_name, envelope)
        .map_err(|error| format!("failed to emit `{event_name}`: {error}"))
}

pub fn emit_theme_settings_changed(app: &AppHandle, active_theme_id: String) -> Result<(), String> {
    emit_window_event(
        app,
        names::THEME_SETTINGS_CHANGED,
        None,
        None,
        ThemeSettingsChangedPayload { active_theme_id },
    )
}

pub fn emit_font_settings_changed(app: &AppHandle, active_font_id: String) -> Result<(), String> {
    emit_window_event(
        app,
        names::FONT_SETTINGS_CHANGED,
        None,
        None,
        FontSettingsChangedPayload { active_font_id },
    )
}

pub fn emit_session_closed(app: &AppHandle, session_id: String) -> Result<(), String> {
    emit_window_event(
        app,
        names::SESSION_CLOSED,
        None,
        Some(session_id.clone()),
        SessionClosedPayload { session_id },
    )
}

pub fn emit_cache_invalidated(
    app: &AppHandle,
    project_cache_id: Option<String>,
    mod_id: Option<String>,
    scene_id: Option<String>,
    source_hash: Option<String>,
    cache_kind: impl Into<String>,
    reason: impl Into<String>,
) -> Result<(), String> {
    emit_window_event(
        app,
        names::CACHE_INVALIDATED,
        None,
        None,
        CacheInvalidatedPayload {
            project_cache_id,
            mod_id,
            scene_id,
            source_hash,
            cache_kind: cache_kind.into(),
            reason: reason.into(),
        },
    )
}

pub fn emit_asset_registry_changed(app: &AppHandle, mod_id: String) -> Result<(), String> {
    emit_window_event(
        app,
        names::ASSET_REGISTRY_CHANGED,
        None,
        None,
        AssetRegistryChangedPayload { mod_id },
    )
}

pub fn emit_asset_descriptor_changed(
    app: &AppHandle,
    mod_id: String,
    asset_key: String,
    descriptor_relative_path: String,
    reason: impl Into<String>,
) -> Result<(), String> {
    emit_window_event(
        app,
        names::ASSET_DESCRIPTOR_CHANGED,
        None,
        None,
        AssetDescriptorChangedPayload {
            mod_id,
            asset_key,
            descriptor_relative_path,
            reason: reason.into(),
        },
    )
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn new_event_id() -> String {
    format!("evt-{}", now_ms())
}
