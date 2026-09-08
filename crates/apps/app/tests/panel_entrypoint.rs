#[path = "../../../runtime/bundles/tests/support/panel_entrypoint.rs"]
mod support;
#[test]
fn panel_client_dispatches_before_app_startup() {
    support::verify(env!("CARGO_BIN_EXE_amigo-app"));
}
