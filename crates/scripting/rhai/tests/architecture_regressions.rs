use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative_path: &str) -> String {
    fs::read_to_string(crate_root().join(relative_path)).expect("file should be readable")
}

#[test]
fn bindings_mod_stays_as_registration_dispatcher() {
    let content = read("src/bindings/mod.rs");
    let line_count = content.lines().count();
    assert!(
        line_count < 180,
        "bindings/mod.rs should stay small; current line count is {line_count}",
    );
    assert!(
        !content.contains(".register_fn("),
        "bindings/mod.rs should delegate registration to binding modules",
    );
    assert!(
        !content.contains("set_main_lens_rain"),
        "domain-specific Rhai functions should live in owner binding modules",
    );
}
