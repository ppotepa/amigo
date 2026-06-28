use std::path::PathBuf;
use std::process::Command;

#[test]
fn app_headless_cli_runs_selected_mod_and_scene() {
    let output = Command::new(env!("CARGO_BIN_EXE_amigo-app"))
        .current_dir(workspace_root())
        .args(["--mod=playground-3d", "--scene=material-lab", "--dev"])
        .output()
        .expect("app binary should run");

    assert!(
        output.status.success(),
        "app should succeed: stdout=`{}` stderr=`{}`",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("root mod: playground-3d"));
    assert!(stdout.contains("active scene: material-lab"));
    assert!(stdout.contains("file watch backend: notify"));
    assert!(stdout.contains("prepared assets:"));
    assert!(stdout.contains("playground-3d/materials/debug-surface (material-3d)"));
}

#[test]
fn app_headless_cli_runs_playground_npr_comic_lines_scene() {
    let output = Command::new(env!("CARGO_BIN_EXE_amigo-app"))
        .current_dir(workspace_root())
        .args(["--mod=playground-npr", "--scene=comic-lines", "--dev"])
        .output()
        .expect("app binary should run");

    assert!(
        output.status.success(),
        "app should succeed: stdout=`{}` stderr=`{}`",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("root mod: playground-npr"));
    assert!(stdout.contains("active scene: comic-lines"));
    assert!(stdout.contains("scene document components: Camera3D x1, CameraController3D x1, InputActionMap x1"));
    assert!(stdout.contains("playground-npr/meshes/soldier (mesh-3d)"));
    assert!(stdout.contains("scene.plugin(amigo.rendering.3d.scene-command.NprPreset3d)"));
    assert!(stdout
        .contains("3d.mesh.apply_npr_preset(playground-npr-model-1-soldier, default_gpu_comic)"));
    assert!(stdout.contains("7 final, 8 kinds, 9 dropout, 0 width/alpha"));
}

#[test]
fn app_headless_cli_rejects_missing_scene() {
    let output = Command::new(env!("CARGO_BIN_EXE_amigo-app"))
        .current_dir(workspace_root())
        .args(["--mod=playground-3d", "--scene=missing-scene"])
        .output()
        .expect("app binary should run");

    assert!(
        !output.status.success(),
        "app should fail for a missing scene"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing-scene"));
    assert!(stderr.contains("playground-3d"));
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}
