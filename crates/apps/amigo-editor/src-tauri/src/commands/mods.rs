use std::fs;
use tauri::State;

use crate::cache::index;
use crate::cache::root::EditorPaths;
use crate::dto::{EditorModDetailsDto, EditorModSummaryDto};
use crate::mods::discovery::{default_mods_root, discover_editor_mods, discovered_mod_ids};
use crate::mods::metadata::{mod_details, mod_summary};

use super::shared::reveal_path;

pub fn list_known_mods(paths: State<'_, EditorPaths>) -> Result<Vec<EditorModSummaryDto>, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_ids = discovered_mod_ids(&discovered);
    for discovered_mod in &discovered {
        let _ = index::upsert_project_index_entry(&paths.cache_root, discovered_mod);
    }
    Ok(discovered
        .iter()
        .map(|discovered_mod| mod_summary(discovered_mod, &discovered_ids))
        .collect())
}

pub fn get_mod_details(
    mod_id: String,
    paths: State<'_, EditorPaths>,
) -> Result<EditorModDetailsDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_ids = discovered_mod_ids(&discovered);
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let _ = index::upsert_project_index_entry(&paths.cache_root, discovered_mod);
    Ok(mod_details(discovered_mod, &discovered_ids))
}

pub fn validate_mod(
    mod_id: String,
    paths: State<'_, EditorPaths>,
) -> Result<EditorModDetailsDto, String> {
    get_mod_details(mod_id, paths)
}

pub fn reveal_mod_folder(mod_id: String) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    reveal_path(&discovered_mod.root_path)?;
    Ok(discovered_mod.root_path.display().to_string())
}

pub fn delete_mod_project(mod_id: String) -> Result<String, String> {
    let lowered = mod_id.to_lowercase();
    if lowered == "core" || lowered == "core-game" || lowered.starts_with("core-") {
        return Err(format!("refusing to delete protected project `{mod_id}`"));
    }

    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

    let mods_root = default_mods_root();
    let canonical_mods_root = mods_root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mods root `{}`: {error}",
            mods_root.display()
        )
    })?;
    let canonical_project_root = discovered_mod.root_path.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize project root `{}`: {error}",
            discovered_mod.root_path.display()
        )
    })?;
    if !canonical_project_root.starts_with(&canonical_mods_root) {
        return Err(format!(
            "refusing to delete `{}` because it is outside mods root `{}`",
            canonical_project_root.display(),
            canonical_mods_root.display()
        ));
    }

    fs::remove_dir_all(&canonical_project_root).map_err(|error| {
        format!(
            "failed to delete `{}`: {error}",
            canonical_project_root.display()
        )
    })?;

    Ok(canonical_project_root.display().to_string())
}
