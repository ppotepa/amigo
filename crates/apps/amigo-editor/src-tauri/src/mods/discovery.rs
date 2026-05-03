use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

use amigo_modding::{DiscoveredMod, ModCatalog};

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};

pub fn default_mods_root() -> PathBuf {
    if let Some(root) = crate::settings::editor_settings::effective_mods_root() {
        return PathBuf::from(root);
    }

    if let Ok(root) = env::var("AMIGO_MODS_ROOT") {
        return PathBuf::from(root);
    }

    let mut current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        let candidate = current.join("mods");
        if candidate.is_dir() {
            return candidate;
        }

        if !current.pop() {
            break;
        }
    }

    PathBuf::from("mods")
}

pub fn discover_editor_mods() -> Result<Vec<DiscoveredMod>, EditorDiagnosticDto> {
    let mods_root = default_mods_root();
    ModCatalog::discover_unresolved(&mods_root).map_err(|error| EditorDiagnosticDto {
        level: DiagnosticLevel::Error,
        code: "mods_discovery_failed".to_owned(),
        message: error.to_string(),
        path: Some(mods_root.display().to_string()),
    })
}

pub fn discovered_mod_ids(mods: &[DiscoveredMod]) -> BTreeSet<String> {
    mods.iter()
        .map(|discovered| discovered.manifest.id.clone())
        .collect()
}
