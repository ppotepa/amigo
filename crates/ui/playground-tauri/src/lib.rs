//! A neutral WebView host. Domain frontend assets are supplied by runtime composition.
use std::{borrow::Cow, collections::BTreeMap};
#[cfg(windows)]
mod native;
use tauri::{Assets, Wry, utils::assets::{AssetKey, CspHash}};

struct ClientAssets(BTreeMap<String, Vec<u8>>);
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
async fn pick_file(filters: Vec<FileFilter>) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    for filter in filters {
        let extensions = filter.extensions.iter().map(String::as_str).collect::<Vec<_>>();
        dialog = dialog.add_filter(&filter.name, &extensions);
    }
    Ok(dialog
        .pick_file()
        .await
        .map(|file| file.path().to_string_lossy().into_owned()))
}

#[derive(Debug, serde::Deserialize)]
struct FileFilter {
    name: String,
    extensions: Vec<String>,
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
            pick_file,
            native::viewport_rect,
            native::frame_consumed,
            native::presentation_failed
        ]);
    #[cfg(not(windows))]
    let builder = builder.invoke_handler(tauri::generate_handler![pick_file]);
    builder
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
