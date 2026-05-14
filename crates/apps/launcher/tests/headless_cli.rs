use std::path::PathBuf;
use std::process::Command;

#[test]
fn launcher_headless_cli_runs_selected_profile_mod_and_scene() {
    let output = Command::new(env!("CARGO_BIN_EXE_amigo-launcher"))
        .current_dir(workspace_root())
        .args([
            "--profile",
            "dev",
            "--mod=playground-2d",
            "--scene=sprite-lab",
            "--headless",
        ])
        .output()
        .expect("launcher binary should run");

    assert!(
        output.status.success(),
        "launcher should succeed: stdout=`{}` stderr=`{}`",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("profile id: dev"));
    assert!(stdout.contains("root mod: playground-2d"));
    assert!(stdout.contains("active scene: sprite-lab"));
    assert!(stdout.contains("file watch backend: notify"));
    assert!(stdout.contains("watched reload targets:"));
}

#[test]
fn launcher_headless_cli_blocks_invalid_scene_before_launch() {
    let output = Command::new(env!("CARGO_BIN_EXE_amigo-launcher"))
        .current_dir(workspace_root())
        .args([
            "--profile",
            "dev",
            "--mod=playground-2d",
            "--scene=missing-scene",
            "--headless",
        ])
        .output()
        .expect("launcher binary should run");

    assert!(
        !output.status.success(),
        "launcher should fail for an invalid scene"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing-scene"));
    assert!(stderr.contains("playground-2d"));
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}
