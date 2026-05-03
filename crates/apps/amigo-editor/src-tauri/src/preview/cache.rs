use std::fs;
use std::path::{Path, PathBuf};

pub fn scene_preview_cache_dir(
    cache_root: &Path,
    project_cache_id: &str,
    scene_id: &str,
    source_hash: &str,
) -> PathBuf {
    cache_root
        .join("projects")
        .join(crate::cache::project_id::safe_path_part(project_cache_id))
        .join("previews")
        .join("scenes")
        .join(crate::cache::project_id::safe_path_part(scene_id))
        .join(source_hash)
}

pub fn ensure_cache_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| {
        format!(
            "failed to create preview cache `{}`: {error}",
            path.display()
        )
    })
}
