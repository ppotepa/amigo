use tauri::State;

use crate::cache::index;
use crate::cache::root::EditorPaths;
use crate::dto::{EditorModDetailsDto, EditorModSummaryDto};
use crate::mods::discovery::{discover_editor_mods, discovered_mod_ids};
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
