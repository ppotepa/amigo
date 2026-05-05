use std::path::{Path, PathBuf};
use tauri::AppHandle;

use crate::dto::{EditorProjectFileContentDto, WriteProjectFileRequestDto};
use crate::events::bus;
use crate::mods::discovery::discover_editor_mods;

use super::project_tree::classify_project_file;
use super::shared::reveal_path;

pub const MAX_TEXT_FILE_BYTES: u64 = 512 * 1024;

pub fn read_project_file(
    mod_id: String,
    relative_path: String,
) -> Result<EditorProjectFileContentDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let path = resolve_project_relative_path(&discovered_mod.root_path, &relative_path)?;
    let metadata = std::fs::metadata(&path)
        .map_err(|error| format!("failed to read metadata `{}`: {error}", path.display()))?;

    if !metadata.is_file() {
        return Err(format!("project path `{relative_path}` is not a file"));
    }

    let kind = classify_project_file(&path, false);
    if !is_readable_project_text_kind(&kind) {
        return Err(format!(
            "project file `{relative_path}` is not a supported text file"
        ));
    }

    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(format!(
            "project file `{relative_path}` is too large to preview as text: {} bytes",
            metadata.len()
        ));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read project file `{}`: {error}", path.display()))?;

    Ok(EditorProjectFileContentDto {
        mod_id,
        path: path.display().to_string(),
        relative_path,
        language: language_for_project_file_kind(&kind).to_owned(),
        kind,
        size_bytes: metadata.len(),
        content,
        diagnostics: Vec::new(),
    })
}

pub fn write_project_file(
    app: AppHandle,
    mod_id: String,
    request: WriteProjectFileRequestDto,
) -> Result<EditorProjectFileContentDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let path = resolve_project_relative_path(&discovered_mod.root_path, &request.relative_path)?;
    let kind = classify_project_file(&path, false);
    if !is_readable_project_text_kind(&kind) {
        return Err(format!(
            "project file `{}` is not a supported text file",
            request.relative_path
        ));
    }
    std::fs::write(&path, request.content.as_bytes())
        .map_err(|error| format!("failed to write project file `{}`: {error}", path.display()))?;

    let relative_path = request.relative_path.clone();
    if relative_path.ends_with(".image.yml")
        || relative_path.ends_with(".sprite.yml")
        || relative_path.ends_with(".atlas.yml")
        || relative_path.ends_with(".tileset.yml")
        || relative_path.ends_with(".tilemap.yml")
    {
        let asset_key = asset_key_from_descriptor_relative_path(&mod_id, &relative_path)
            .unwrap_or_else(|| relative_path.clone());
        let _ = bus::emit_asset_descriptor_changed(
            &app,
            mod_id.clone(),
            asset_key,
            relative_path.clone(),
            "saved",
        );
        let _ = bus::emit_asset_registry_changed(&app, mod_id.clone());
        let _ = bus::emit_cache_invalidated(
            &app,
            None,
            Some(mod_id.clone()),
            None,
            None,
            "asset",
            "project text file saved",
        );
    }

    read_project_file(mod_id, relative_path)
}

pub fn reveal_project_file(mod_id: String, relative_path: String) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let path = resolve_project_relative_path(&discovered_mod.root_path, &relative_path)?;
    reveal_path(&path)?;
    Ok(path.display().to_string())
}

pub fn create_expected_project_folder(
    mod_id: String,
    expected_path: String,
) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

    let normalized = expected_path.trim().replace('\\', "/");
    let allowed = [
        "scenes/",
        "raw/",
        "raw/images/",
        "raw/audio/",
        "raw/fonts/",
        "raw/external/",
        "spritesheets/",
        "audio/",
        "fonts/",
        "scripts/",
        "scripts/components/",
        "scripts/packages/",
        "data/",
        "docs/",
        "custom/",
    ];
    if !allowed.contains(&normalized.as_str()) {
        return Err(format!(
            "expected project folder `{expected_path}` is not part of the current editor structure contract"
        ));
    }

    let relative_path = normalized.trim_end_matches('/');
    let target = discovered_mod.root_path.join(relative_path);
    let canonical_root = discovered_mod.root_path.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            discovered_mod.root_path.display()
        )
    })?;
    let parent = target
        .parent()
        .ok_or_else(|| format!("project folder `{expected_path}` has no parent"))?;
    std::fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create parent directory `{}`: {error}",
            parent.display()
        )
    })?;
    let canonical_parent = parent.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize parent directory `{}`: {error}",
            parent.display()
        )
    })?;
    if !canonical_parent.starts_with(&canonical_root) {
        return Err(format!("project folder `{expected_path}` escapes mod root"));
    }

    std::fs::create_dir_all(&target).map_err(|error| {
        format!(
            "failed to create project folder `{}`: {error}",
            target.display()
        )
    })?;
    Ok(target.display().to_string())
}

pub fn resolve_project_relative_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let candidate = root.join(relative_path);
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            root.display()
        )
    })?;
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize project path `{}`: {error}",
            candidate.display()
        )
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!("project path `{relative_path}` escapes mod root"));
    }

    Ok(canonical_candidate)
}

pub fn is_readable_project_text_kind(kind: &str) -> bool {
    matches!(
        kind,
        "manifest"
            | "sceneDocument"
            | "sceneScript"
            | "script"
            | "scriptPackage"
            | "tileset"
            | "tilemap"
            | "imageAsset"
            | "font"
            | "audio"
            | "particle"
            | "material"
            | "ui"
            | "input"
            | "yaml"
    )
}

pub fn language_for_project_file_kind(kind: &str) -> &'static str {
    match kind {
        "manifest" => "toml",
        "sceneDocument" | "scriptPackage" | "tileset" | "tilemap" | "imageAsset" | "font"
        | "audio" | "particle" | "material" | "ui" | "input" | "yaml" => "yaml",
        "sceneScript" | "script" => "rhai",
        _ => "text",
    }
}

pub fn asset_key_from_descriptor_relative_path(mod_id: &str, relative_path: &str) -> Option<String> {
    let normalized = relative_path.replace('\\', "/");
    if normalized.starts_with("spritesheets/") {
        if normalized.ends_with("/spritesheet.yml") || normalized.ends_with("/spritesheet.yaml") {
            let spritesheet_id = normalized.split('/').nth(1)?;
            return Some(format!("{mod_id}/spritesheets/{spritesheet_id}"));
        }
        for marker in ["/tilesets/", "/rulesets/", "/animations/"] {
            if normalized.contains(marker) {
                return Some(format!(
                    "{mod_id}/{}",
                    normalized
                        .trim_end_matches(".yml")
                        .trim_end_matches(".yaml")
                ));
            }
        }
    }
    if normalized.starts_with("fonts/") && normalized.ends_with("/font.yml") {
        let font_id = normalized.split('/').nth(1)?;
        return Some(format!("{mod_id}/fonts/{font_id}"));
    }
    if normalized.starts_with("audio/") && normalized.ends_with("/audio.yml") {
        let audio_id = normalized.split('/').nth(1)?;
        return Some(format!("{mod_id}/audio/{audio_id}"));
    }
    if normalized.starts_with("data/tilemaps/")
        && (normalized.ends_with(".tilemap.yml") || normalized.ends_with(".tilemap.yaml"))
    {
        let asset_id = normalized
            .trim_start_matches("data/tilemaps/")
            .trim_end_matches(".yml")
            .trim_end_matches(".yaml")
            .trim_end_matches(".tilemap");
        return Some(format!("{mod_id}/data/tilemaps/{asset_id}"));
    }
    None
}
