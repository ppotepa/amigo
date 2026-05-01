use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use amigo_core::AmigoResult;
use amigo_modding::ModCatalog;
use crate::config::LauncherConfig;
use toml;

use super::{LauncherManifestMetadata, LauncherMetadata, KnownMod};

pub(super) fn discover_known_mods(config: &LauncherConfig) -> AmigoResult<Vec<KnownMod>> {
    let discovered = ModCatalog::discover_unresolved(Path::new(&config.mods_root))?;
    let mut known_ids = BTreeSet::new();
    let mut known_mods = Vec::new();

    for discovered_mod in discovered {
        let launcher_metadata = read_launcher_metadata(&discovered_mod.root_path);
        known_ids.insert(discovered_mod.manifest.id.clone());
        known_mods.push(KnownMod {
            id: discovered_mod.manifest.id.clone(),
            name: discovered_mod.manifest.name.clone(),
            description: discovered_mod
                .manifest
                .description
                .clone()
                .unwrap_or_default(),
            scenes: discovered_mod
                .manifest
                .scenes
                .iter()
                .filter(|scene| scene.is_launcher_visible())
                .cloned()
                .collect(),
            launcher_category: launcher_metadata.category,
            launcher_scene_categories: launcher_metadata.scene_categories,
            discovered: true,
        });
    }

    let mut configured_only_ids = BTreeSet::new();

    for profile in &config.profiles {
        if let Some(root_mod) = profile.root_mod.as_deref() {
            if !known_ids.contains(root_mod) {
                configured_only_ids.insert(root_mod.to_owned());
            }
        }
    }

    for mod_id in configured_only_ids {
        known_mods.push(KnownMod {
            id: mod_id.clone(),
            name: mod_id.clone(),
            description: "Configured in launcher profile but not discovered on disk.".to_owned(),
            scenes: Vec::new(),
            launcher_category: None,
            launcher_scene_categories: BTreeMap::new(),
            discovered: false,
        });
    }

    known_mods.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(known_mods)
}

fn read_launcher_metadata(mod_root: &Path) -> LauncherMetadata {
    let raw = match std::fs::read_to_string(mod_root.join("mod.toml")) {
        Ok(raw) => raw,
        Err(_) => return LauncherMetadata::default(),
    };
    let parsed = match toml::from_str::<LauncherManifestMetadata>(&raw) {
        Ok(parsed) => parsed,
        Err(_) => return LauncherMetadata::default(),
    };
    let scene_categories = parsed
        .scenes
        .into_iter()
        .filter_map(|scene| {
            let category = normalize_launcher_category(&scene.launcher_category)?;
            if scene.id.trim().is_empty() {
                return None;
            }
            Some((scene.id, category))
        })
        .collect();
    LauncherMetadata {
        category: normalize_launcher_category(&parsed.launcher_category),
        scene_categories,
    }
}

fn normalize_launcher_category(category: &[String]) -> Option<Vec<String>> {
    let category = category
        .iter()
        .map(|segment| segment.trim())
        .filter(|segment| !segment.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if category.is_empty() {
        None
    } else {
        Some(category)
    }
}
