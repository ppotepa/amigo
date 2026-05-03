use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIdentity {
    pub project_cache_id: String,
    pub created_by: String,
    pub created_at: String,
}

pub fn project_identity_path(mod_root: &Path) -> PathBuf {
    mod_root.join(".amigo-editor").join("project-id.json")
}

pub fn read_project_identity(mod_root: &Path) -> Option<ProjectIdentity> {
    let text = fs::read_to_string(project_identity_path(mod_root)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn project_cache_id_for_root(mod_root: &Path) -> String {
    read_project_identity(mod_root)
        .map(|identity| safe_path_part(&identity.project_cache_id))
        .unwrap_or_else(|| fallback_project_cache_id(mod_root))
}

fn fallback_project_cache_id(mod_root: &Path) -> String {
    let canonical = mod_root
        .canonicalize()
        .unwrap_or_else(|_| mod_root.to_path_buf());
    let mut hasher = DefaultHasher::new();
    "amigo-editor-project-cache-id-v1".hash(&mut hasher);
    canonical.display().to_string().hash(&mut hasher);
    format!("path-{:016x}", hasher.finish())
}

pub fn safe_path_part(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
