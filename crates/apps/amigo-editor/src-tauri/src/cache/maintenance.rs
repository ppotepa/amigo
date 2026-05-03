use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::cache::index::{load_project_index, project_dir, projects_dir};
use crate::cache::policies::load_cache_policy;
use crate::dto::CacheMaintenanceResultDto;

#[derive(Debug)]
struct PreviewEntry {
    path: PathBuf,
    size_bytes: u64,
    modified: SystemTime,
}

pub fn run_cache_maintenance(root: &Path) -> Result<CacheMaintenanceResultDto, String> {
    let policy = load_cache_policy(root);
    let mut entries = collect_preview_entries(root)?;
    let mut removed_entries = 0usize;
    let mut removed_bytes = 0u64;

    if let Some(max_age_days) = policy.max_age_days {
        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(max_age_days as u64 * 24 * 60 * 60))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        entries.retain(|entry| {
            if entry.modified < cutoff {
                if fs::remove_dir_all(&entry.path).is_ok() {
                    removed_entries += 1;
                    removed_bytes += entry.size_bytes;
                }
                false
            } else {
                true
            }
        });
    }

    if let Some(limit) = policy.max_preview_cache_bytes {
        let mut total = entries.iter().map(|entry| entry.size_bytes).sum::<u64>();
        entries.sort_by_key(|entry| entry.modified);

        for entry in entries {
            if total <= limit {
                break;
            }

            if fs::remove_dir_all(&entry.path).is_ok() {
                total = total.saturating_sub(entry.size_bytes);
                removed_entries += 1;
                removed_bytes += entry.size_bytes;
            }
        }
    }

    let remaining_preview_bytes = collect_preview_entries(root)?
        .iter()
        .map(|entry| entry.size_bytes)
        .sum();

    Ok(CacheMaintenanceResultDto {
        removed_entries,
        removed_bytes,
        remaining_preview_bytes,
        orphaned_projects_removed: 0,
    })
}

pub fn clear_orphaned_project_caches(root: &Path) -> Result<CacheMaintenanceResultDto, String> {
    let index = load_project_index(root);
    let mut removed = 0usize;
    let mut removed_bytes = 0u64;

    for project in index.projects {
        if Path::new(&project.last_known_root_path).exists() {
            continue;
        }

        let path = project_dir(root, &project.project_cache_id);
        if !path.exists() {
            continue;
        }

        let size = path_size(&path).unwrap_or(0);
        fs::remove_dir_all(&path)
            .map_err(|error| format!("failed to remove orphaned cache `{}`: {error}", path.display()))?;
        removed += 1;
        removed_bytes += size;
    }

    Ok(CacheMaintenanceResultDto {
        removed_entries: removed,
        removed_bytes,
        remaining_preview_bytes: collect_preview_entries(root)?
            .iter()
            .map(|entry| entry.size_bytes)
            .sum(),
        orphaned_projects_removed: removed,
    })
}

fn collect_preview_entries(root: &Path) -> Result<Vec<PreviewEntry>, String> {
    let mut entries = Vec::new();
    let projects_path = projects_dir(root);
    if !projects_path.exists() {
        return Ok(entries);
    }

    for project in fs::read_dir(&projects_path)
        .map_err(|error| format!("failed to read projects cache `{}`: {error}", projects_path.display()))?
    {
        let project = project.map_err(|error| format!("failed to read project cache entry: {error}"))?;
        let scene_root = project.path().join("previews").join("scenes");
        collect_hash_dirs(&scene_root, &mut entries)?;
    }

    Ok(entries)
}

fn collect_hash_dirs(path: &Path, entries: &mut Vec<PreviewEntry>) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    for scene in fs::read_dir(path)
        .map_err(|error| format!("failed to read preview scene cache `{}`: {error}", path.display()))?
    {
        let scene = scene.map_err(|error| format!("failed to read scene preview cache entry: {error}"))?;
        if !scene.path().is_dir() {
            continue;
        }

        for hash_dir in fs::read_dir(scene.path())
            .map_err(|error| format!("failed to read preview hash cache: {error}"))?
        {
            let hash_dir = hash_dir.map_err(|error| format!("failed to read preview hash entry: {error}"))?;
            let path = hash_dir.path();
            if !path.is_dir() {
                continue;
            }

            let metadata = fs::metadata(&path)
                .map_err(|error| format!("failed to read preview cache metadata `{}`: {error}", path.display()))?;
            entries.push(PreviewEntry {
                size_bytes: path_size(&path).unwrap_or(0),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                path,
            });
        }
    }

    Ok(())
}

fn path_size(path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|error| {
        format!("failed to read metadata for cache entry `{}`: {error}", path.display())
    })?;

    if metadata.is_file() {
        return Ok(metadata.len());
    }

    if !metadata.is_dir() {
        return Ok(0);
    }

    let mut sum = 0u64;
    for entry in fs::read_dir(path)
        .map_err(|error| format!("failed to read cache directory `{}`: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read cache entry: {error}"))?;
        sum += path_size(&entry.path())?;
    }
    Ok(sum)
}
