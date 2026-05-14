use super::*;

#[test]
fn bootstrap_options_editor_mode_defaults_false() {
    let options = BootstrapOptions::default();
    assert!(!options.editor_mode);
}

#[test]
fn bootstrap_options_editor_mode_can_be_enabled() {
    let options = BootstrapOptions::default().with_editor_mode(true);
    assert!(options.editor_mode);
}
