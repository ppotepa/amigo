#[path = "../../../runtime/bundles/tests/support/panel_entrypoint.rs"]
mod support;
#[test]
fn panel_client_dispatches_before_launcher_config_or_tui() {
    support::verify(env!("CARGO_BIN_EXE_amigo-launcher"));
}
