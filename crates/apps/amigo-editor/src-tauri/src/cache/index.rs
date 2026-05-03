use std::fs;
use std::path::{Path, PathBuf};

use amigo_modding::DiscoveredMod;
use serde::Serialize;
use serde_json::Value;

use crate::cache::project_id::project_cache_id_for_root;
use crate::dto::{
    CacheInfoDto, CacheProjectInfoDto, ProjectAliasesDto, ProjectIndexDto, ProjectIndexEntryDto,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectCacheMetadata<'a> {
    project_cache_id: &'a str,
    mod_id: &'a str,
    display_name: &'a str,
    root_path: &'a str,
    last_seen_at: &'a str,
}

pub fn project_index_path(root: &Path) -> PathBuf {
    root.join("project-index.json")
}

pub fn load_project_index(root: &Path) -> ProjectIndexDto {
    let path = project_index_path(root);
    let Ok(text) = fs::read_to_string(path) else {
        return ProjectIndexDto::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_project_index(root: &Path, index: &ProjectIndexDto) -> Result<(), String> {
    fs::create_dir_all(root)
        .map_err(|error| format!("failed to create cache root `{}`: {error}", root.display()))?;
    let text = serde_json::to_string_pretty(index)
        .map_err(|error| format!("failed to serialize project index: {error}"))?;
    fs::write(project_index_path(root), text)
        .map_err(|error| format!("failed to write project index: {error}"))
}

pub fn upsert_project_index_entry(root: &Path, discovered: &DiscoveredMod) -> Result<(), String> {
    let project_cache_id = project_cache_id_for_root(&discovered.root_path);
    let display_name = discovered.manifest.name.clone();
    let mod_id = discovered.manifest.id.clone();
    let root_path = discovered.root_path.display().to_string();
    let last_seen_at = unix_seconds();

    let mut index = load_project_index(root);
    let entry = index
        .projects
        .iter_mut()
        .find(|entry| entry.project_cache_id == project_cache_id);

    match entry {
        Some(entry) => {
            entry.last_known_display_name = display_name.clone();
            entry.last_known_mod_id = mod_id.clone();
            entry.last_known_root_path = root_path.clone();
            entry.last_seen_at = last_seen_at.clone();
            push_unique(&mut entry.aliases.display_names, display_name);
            push_unique(&mut entry.aliases.mod_ids, mod_id);
            push_unique(&mut entry.aliases.root_paths, root_path);
        }
        None => {
            index.projects.push(ProjectIndexEntryDto {
                project_cache_id: project_cache_id.clone(),
                last_known_display_name: display_name.clone(),
                last_known_mod_id: mod_id.clone(),
                last_known_root_path: root_path.clone(),
                last_seen_at: last_seen_at.clone(),
                aliases: ProjectAliasesDto {
                    display_names: vec![display_name],
                    mod_ids: vec![mod_id],
                    root_paths: vec![root_path],
                },
            });
        }
    }

    write_project_cache_meta(
        root,
        &project_cache_id,
        &discovered.manifest.id,
        &discovered.manifest.name,
        &discovered.root_path.display().to_string(),
    )?;
    save_project_index(root, &index)
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn path_size(path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|error| {
        format!(
            "failed to read metadata for cache entry `{}`: {error}",
            path.display()
        )
    })?;

    if metadata.is_file() {
        return Ok(metadata.len());
    }

    if !metadata.is_dir() {
        return Ok(0);
    }

    let mut sum = 0u64;
    for entry in fs::read_dir(path).map_err(|error| {
        format!(
            "failed to read cache directory `{}`: {error}",
            path.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read cache entry in `{}`: {error}",
                path.display()
            )
        })?;
        sum += path_size(&entry.path())?;
    }
    Ok(sum)
}

pub fn projects_dir(root: &Path) -> PathBuf {
    root.join("projects")
}

pub fn project_dir(root: &Path, project_cache_id: &str) -> PathBuf {
    projects_dir(root).join(project_cache_id)
}

pub fn previews_dir(root: &Path, project_cache_id: &str) -> PathBuf {
    project_dir(root, project_cache_id).join("previews")
}

pub fn clear_project_cache(root: &Path, project_cache_id: &str) -> Result<(), String> {
    let path = project_dir(root, project_cache_id);
    if !path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&path).map_err(|error| {
        format!(
            "failed to clear project cache `{}`: {error}",
            path.display()
        )
    })
}

pub fn clear_preview_cache(root: &Path, project_cache_id: &str) -> Result<(), String> {
    let path = previews_dir(root, project_cache_id);
    if !path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&path).map_err(|error| {
        format!("failed to clear preview cache for `{project_cache_id}`: {error}")
    })?;
    fs::create_dir_all(&path).map_err(|error| {
        format!(
            "failed to recreate preview directory `{}`: {error}",
            path.display()
        )
    })?;
    Ok(())
}

pub fn collect_cache_info(root: &Path, cache_root_mode: &str) -> Result<CacheInfoDto, String> {
    let total_size_bytes = path_size(root).unwrap_or(0);
    let mut projects = Vec::new();
    let index = load_project_index(root);

    for entry in &index.projects {
        let project_path = project_dir(root, &entry.project_cache_id);
        let project_size_bytes = if project_path.exists() {
            path_size(&project_path).unwrap_or(0)
        } else {
            0
        };

        projects.push(CacheProjectInfoDto {
            project_cache_id: entry.project_cache_id.clone(),
            mod_id: entry.last_known_mod_id.clone(),
            display_name: entry.last_known_display_name.clone(),
            root_path: entry.last_known_root_path.clone(),
            last_seen_at: entry.last_seen_at.clone(),
            project_size_bytes,
        });
    }

    let projects_path = projects_dir(root);
    if projects_path.exists() {
        for entry in fs::read_dir(&projects_path).map_err(|error| {
            format!(
                "failed to read cache project directory `{}`: {error}",
                projects_path.display()
            )
        })? {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read cache project entry in `{}`: {error}",
                    projects_path.display()
                )
            })?;
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }

            let Some(project_cache_id) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };

            if projects
                .iter()
                .any(|project| project.project_cache_id == project_cache_id)
            {
                continue;
            }

            let (mod_id, display_name, root_path, last_seen_at) =
                read_project_cache_meta(&entry_path).unwrap_or((
                    "unknown-mod".to_owned(),
                    "Unknown Mod".to_owned(),
                    "unknown-root".to_owned(),
                    "unknown".to_owned(),
                ));
            let project_size_bytes = path_size(&entry_path).unwrap_or(0);

            projects.push(CacheProjectInfoDto {
                project_cache_id,
                mod_id,
                display_name,
                root_path,
                last_seen_at,
                project_size_bytes,
            });
        }
    }

    projects.sort_by(|left, right| right.last_seen_at.cmp(&left.last_seen_at));

    Ok(CacheInfoDto {
        cache_root: root.display().to_string(),
        cache_root_mode: cache_root_mode.to_owned(),
        total_size_bytes,
        project_count: projects.len(),
        projects,
    })
}

pub fn write_project_cache_meta(
    root: &Path,
    project_cache_id: &str,
    mod_id: &str,
    display_name: &str,
    root_path: &str,
) -> Result<(), String> {
    let meta_path = project_dir(root, project_cache_id).join("project-cache.json");
    if let Some(parent) = meta_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create cache project directory `{}`: {error}",
                parent.display()
            )
        })?;
    }

    let last_seen_at = unix_seconds();
    let metadata = ProjectCacheMetadata {
        project_cache_id,
        mod_id,
        display_name,
        root_path,
        last_seen_at: &last_seen_at,
    };

    let text = serde_json::to_string_pretty(&metadata)
        .map_err(|error| format!("failed to serialize project cache metadata: {error}"))?;
    fs::write(&meta_path, text).map_err(|error| {
        format!(
            "failed to write project cache metadata `{}`: {error}",
            meta_path.display()
        )
    })
}

pub fn unix_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn read_project_cache_meta(project_root: &Path) -> Option<(String, String, String, String)> {
    let metadata_path = project_root.join("project-cache.json");
    if !metadata_path.is_file() {
        return None;
    }

    let text = fs::read_to_string(&metadata_path).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let mod_id = value
        .get("modId")
        .and_then(Value::as_str)
        .unwrap_or("unknown-mod");
    let display_name = value
        .get("displayName")
        .and_then(Value::as_str)
        .unwrap_or(mod_id);
    let root_path = value
        .get("rootPath")
        .and_then(Value::as_str)
        .unwrap_or("unknown-root");
    let last_seen_at = value
        .get("lastSeenAt")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Some((
        mod_id.to_owned(),
        display_name.to_owned(),
        root_path.to_owned(),
        last_seen_at.to_owned(),
    ))
}
