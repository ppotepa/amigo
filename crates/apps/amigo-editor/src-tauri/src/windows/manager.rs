use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use super::descriptors::EditorWindowKind;

pub fn open_or_focus_window(app: &AppHandle, kind: EditorWindowKind) -> Result<(), String> {
    let app = app.clone();
    let label = kind.label();

    app.clone()
        .run_on_main_thread(move || {
            if let Err(error) = open_or_focus_window_now(&app, kind) {
                eprintln!("failed to open Amigo Editor window `{label}`: {error}");
            }
        })
        .map_err(|error| error.to_string())
}

fn open_or_focus_window_now(
    app: &AppHandle,
    kind: EditorWindowKind,
) -> Result<WebviewWindow, String> {
    let label = kind.label();

    if let Some(existing) = app.get_webview_window(&label) {
        existing.show().map_err(|error| error.to_string())?;
        existing.set_focus().map_err(|error| error.to_string())?;
        return Ok(existing);
    }

    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(kind.route().into()))
        .title(kind.title())
        .visible(false);

    let builder = match &kind {
        EditorWindowKind::Startup => builder
            .inner_size(1340.0, 880.0)
            .min_inner_size(1340.0, 880.0)
            .max_inner_size(1340.0, 880.0)
            .resizable(false)
            .maximizable(false)
            .center(),
        EditorWindowKind::Workspace { .. } => builder
            .inner_size(1440.0, 900.0)
            .min_inner_size(1200.0, 720.0)
            .resizable(true)
            .maximizable(true)
            .fullscreen(false)
            .center(),
        EditorWindowKind::Theme => builder
            .inner_size(1320.0, 940.0)
            .min_inner_size(1100.0, 760.0)
            .resizable(true)
            .maximizable(false)
            .center(),
        EditorWindowKind::Settings => builder
            .inner_size(1080.0, 780.0)
            .min_inner_size(900.0, 640.0)
            .resizable(true)
            .maximizable(false)
            .center(),
        EditorWindowKind::ModSettings { .. } => builder
            .inner_size(1080.0, 820.0)
            .min_inner_size(900.0, 680.0)
            .resizable(true)
            .maximizable(false)
            .center(),
    };

    let window = builder.build().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(window)
}
