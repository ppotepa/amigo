use std::fs;
use std::path::{Path, PathBuf};

use amigo_modding::ModSceneManifest;

const ASSET_INPUT_DIRS: &[&str] = &[
    "textures",
    "sprites",
    "spritesheets",
    "tilesets",
    "tilemaps",
    "audio",
];

pub fn collect_preview_inputs(mod_root: &Path, scene: &ModSceneManifest) -> Vec<PathBuf> {
    let mut inputs = vec![
        mod_root.join("mod.toml"),
        scene.document_path(mod_root),
        scene.script_path(mod_root),
    ];

    for dir_name in ASSET_INPUT_DIRS {
        collect_files(&mod_root.join(dir_name), &mut inputs);
    }

    inputs.sort();
    inputs.dedup();
    inputs
}

pub fn relative_input_paths(mod_root: &Path, inputs: &[PathBuf]) -> Vec<String> {
    inputs
        .iter()
        .map(|path| {
            path.strip_prefix(mod_root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect()
}

fn collect_files(path: &Path, inputs: &mut Vec<PathBuf>) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

    if metadata.is_file() {
        inputs.push(path.to_path_buf());
        return;
    }

    if !metadata.is_dir() {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        collect_files(&entry.path(), inputs);
    }
}
