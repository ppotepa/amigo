//! A neutral WebView host. Domain frontend assets are supplied by runtime composition.
use std::{borrow::Cow, collections::BTreeMap, sync::atomic::{AtomicU64, Ordering}};
#[cfg(windows)]
mod native;
use tauri::{
    AppHandle, Assets, Emitter, Manager, Wry,
    utils::assets::{AssetKey, CspHash},
};

struct ClientAssets(BTreeMap<String, Vec<u8>>);
static BRUSH_EDITOR_SESSION: AtomicU64 = AtomicU64::new(0);
impl Assets<Wry> for ClientAssets {
    fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
        self.0
            .get(key.as_ref().trim_start_matches('/'))
            .map(|v| Cow::Borrowed(v.as_slice()))
    }
    fn iter(&self) -> Box<tauri::utils::assets::AssetsIter<'_>> {
        Box::new(
            self.0
                .iter()
                .map(|(k, v)| (Cow::Borrowed(k.as_str()), Cow::Borrowed(v.as_slice()))),
        )
    }
    fn csp_hashes(&self, _: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
        Box::new(std::iter::empty())
    }
}

#[tauri::command]
async fn pick_model_file() -> Result<Option<String>, String> {
    Ok(rfd::AsyncFileDialog::new()
        .add_filter("3D model", &["glb", "gltf"])
        .pick_file()
        .await
        .map(|file| file.path().to_string_lossy().into_owned()))
}

/// Opens the brush editor as a child of the single playground window. The
/// editor has no engine connection; edits are exchanged through the parent
/// session by the frontend and the child is reused when opened repeatedly.
#[tauri::command]
fn open_brush_editor(app: AppHandle, draft: serde_json::Value) -> Result<u64, String> {
    validate_brush_draft(&draft)?;
    let session_id = BRUSH_EDITOR_SESSION.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
    let payload = serde_json::json!({"draft": draft, "session_id": session_id, "request_id": 0});
    if let Some(window) = app.get_webview_window("brush-editor") {
        window.emit("brush-draft", &payload).map_err(|e| e.to_string())?;
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(session_id);
    }
    let parent = app
        .get_webview_window("playground")
        .ok_or("playground window is not ready")?;
    let initialization = format!("window.__BRUSH_DRAFT__ = {};", serde_json::to_string(&payload).map_err(|e| e.to_string())?);
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "brush-editor",
        tauri::WebviewUrl::App("index.html?brush-editor=1".into()),
    )
    .title("Brush Editor")
    .inner_size(620.0, 720.0)
    .min_inner_size(420.0, 480.0)
    .initialization_script(&initialization)
    .parent(&parent)
    .map_err(|e| e.to_string())?
    .build()
    .map_err(|e| e.to_string())?;
    let _ = window.set_focus();
    Ok(session_id)
}

#[tauri::command]
#[allow(non_snake_case)]
fn brush_editor_apply(app: AppHandle, draft: serde_json::Value, sessionId: u64, requestId: u64) -> Result<(), String> {
    if !session_is_active(sessionId, BRUSH_EDITOR_SESSION.load(Ordering::Relaxed)) {
        return Err("brush editor session is no longer active".into());
    }
    validate_brush_draft(&draft)?;
    app.emit_to("playground", "brush-apply", serde_json::json!({"draft": draft, "session_id": sessionId, "request_id": requestId})).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
fn brush_editor_cancel(app: AppHandle, sessionId: u64) -> Result<(), String> {
    if !session_is_active(sessionId, BRUSH_EDITOR_SESSION.load(Ordering::Relaxed)) {
        return Err("brush editor session is no longer active".into());
    }
    app.emit_to("playground", "brush-cancel", serde_json::json!({"session_id": sessionId})).map_err(|e| e.to_string())
}

fn session_is_active(session_id: u64, active_session: u64) -> bool {
    session_id != 0 && session_id == active_session
}

fn validate_brush_draft(draft: &serde_json::Value) -> Result<(), String> {
    let object = draft.as_object().ok_or("brush draft must be an object")?;
    let reference = object
        .get("brush")
        .and_then(serde_json::Value::as_object)
        .and_then(|brush| brush.get("brush"))
        .and_then(serde_json::Value::as_object)
        .ok_or("brush draft is missing a pinned brush reference")?;
    let id = reference.get("id").and_then(serde_json::Value::as_str).unwrap_or_default();
    let version = reference.get("version").and_then(serde_json::Value::as_u64).unwrap_or(0);
    if id.trim().is_empty() || version == 0 {
        return Err("brush draft has an invalid pinned brush reference".into());
    }
    for field in ["width", "irregularity", "dryness"] {
        if let Some(value) = object.get(field) {
            let number = value.as_f64().ok_or_else(|| format!("brush draft field `{field}` must be numeric"))?;
            let maximum = if field == "width" { 100.0 } else { 1.0 };
            if !number.is_finite() || !(0.0..=maximum).contains(&number) {
                return Err(format!("brush draft field `{field}` is outside the supported range"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{session_is_active, validate_brush_draft};

    #[test]
    fn brush_draft_validation_requires_pinned_reference_and_finite_values() {
        let valid = serde_json::json!({"brush":{"brush":{"id":"ink-liner","version":1}},"width":2.0});
        assert!(validate_brush_draft(&valid).is_ok());
        assert!(validate_brush_draft(&serde_json::json!({"width":2.0})).is_err());
        assert!(validate_brush_draft(&serde_json::json!({"brush":{"brush":{"id":"ink-liner","version":1}},"width":"bad"})).is_err());
    }

    #[test]
    fn stale_or_empty_editor_sessions_are_rejected() {
        assert!(!session_is_active(0, 1));
        assert!(session_is_active(7, 7));
        assert!(!session_is_active(6, 7));
        assert!(!session_is_active(7, 0));
    }
}

pub fn run(
    label: String,
    mut bootstrap: serde_json::Value,
    native_bootstrap: Option<amigo_playground_native::NativeBootstrap>,
    assets: Vec<(&'static str, &'static [u8])>,
) -> Result<(), String> {
    #[cfg(windows)]
    let native = native_bootstrap
        .ok_or_else(|| "Native companion channel is unavailable".to_string())
        .and_then(amigo_playground_native::native_client);
    #[cfg(windows)]
    {
        bootstrap["native_error"] = native
            .as_ref()
            .err()
            .map_or(serde_json::Value::Null, |error| serde_json::json!(error));
    }
    #[cfg(not(windows))]
    {
        let _ = native_bootstrap;
        bootstrap["native_error"] = serde_json::json!(
            "Native GPU and Local RGBA are currently implemented only on Windows"
        );
    }
    let endpoint = bootstrap
        .get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or("missing session endpoint")?;
    let url = tauri::Url::parse(endpoint).map_err(|e| e.to_string())?;
    if url.scheme() != "ws"
        || url.host_str() != Some("127.0.0.1")
        || url.port().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("session endpoint must be loopback WebSocket".into());
    }
    let mut context = tauri::generate_context!();
    let csp = format!(
        "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' blob: data:; connect-src ipc: http://ipc.localhost {endpoint}; font-src 'self'"
    );
    context.config_mut().app.security.csp = Some(tauri::utils::config::Csp::Policy(csp));
    let _ = context.set_assets(Box::new(ClientAssets(
        assets
            .into_iter()
            .map(|(k, v)| (k.into(), v.to_vec()))
            .collect(),
    )));
    let initialization = format!(
        "Object.defineProperty(window, '__PLAYGROUND__', {{value: {}, writable: false}});",
        serde_json::to_string(&bootstrap).map_err(|e| e.to_string())?
    );
    let builder = tauri::Builder::default();
    #[cfg(windows)]
    let builder = builder
        .manage(native::NativeSession(native.ok()))
        .invoke_handler(tauri::generate_handler![
            pick_model_file,
            open_brush_editor,
            brush_editor_apply,
            brush_editor_cancel,
            native::viewport_rect,
            native::frame_consumed,
            native::presentation_failed
        ]);
    #[cfg(not(windows))]
    let builder =
        builder.invoke_handler(tauri::generate_handler![pick_model_file, open_brush_editor, brush_editor_apply, brush_editor_cancel]);
    builder
        .on_window_event(|window, event| {
            if window.label() == "brush-editor"
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                let session_id = BRUSH_EDITOR_SESSION.load(Ordering::Relaxed);
                if session_id != 0 {
                    let _ = window.app_handle().emit_to(
                        "playground",
                        "brush-cancel",
                        serde_json::json!({"session_id": session_id}),
                    );
                    BRUSH_EDITOR_SESSION.store(0, Ordering::Relaxed);
                }
            }
            if window.label() == "playground"
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                if let Some(child) = window.app_handle().get_webview_window("brush-editor") {
                    let _ = child.close();
                }
            }
        })
        .setup(move |app| {
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "playground",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title(label.clone())
            .inner_size(1440.0, 900.0)
            .min_inner_size(640.0, 480.0)
            .initialization_script(&initialization)
            .on_navigation(|url| {
                matches!(
                    (url.scheme(), url.host_str()),
                    ("http", Some("tauri.localhost")) | ("tauri", Some("localhost"))
                )
            })
            .build()?;
            #[cfg(windows)]
            native::attach(&window)?;
            Ok(())
        })
        .run(context)
        .map_err(|e| e.to_string())
}
