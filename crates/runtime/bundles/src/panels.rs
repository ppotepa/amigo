pub fn enable_external_panels(
    runtime: &amigo_runtime::Runtime,
    executable: std::path::PathBuf,
) -> amigo_core::AmigoResult<()> {
    runtime
        .required::<amigo_panels::PanelService>()?
        .enable_host(executable);
    Ok(())
}
/// Must run before host configuration, console/TUI initialization or logging.
/// stdout belongs exclusively to the framed panel protocol in this mode.
pub fn dispatch_external_panel_client() -> Option<amigo_core::AmigoResult<()>> {
    (std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--runtime-panel-client")))
        .then(run_external_panel_client)
}

fn run_external_panel_client() -> amigo_core::AmigoResult<()> {
    amigo_panel_egui::run().map_err(amigo_core::AmigoError::Message)
}
