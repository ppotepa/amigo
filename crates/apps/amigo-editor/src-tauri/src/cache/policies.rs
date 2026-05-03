use std::fs;
use std::path::Path;

use crate::dto::CachePolicyDto;

const CACHE_POLICY_FILE: &str = "cache-policy.json";

pub fn load_cache_policy(root: &Path) -> CachePolicyDto {
    let path = root.join(CACHE_POLICY_FILE);
    let Ok(text) = fs::read_to_string(path) else {
        return CachePolicyDto::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_cache_policy(root: &Path, policy: &CachePolicyDto) -> Result<(), String> {
    fs::create_dir_all(root)
        .map_err(|error| format!("failed to create cache root `{}`: {error}", root.display()))?;
    let text = serde_json::to_string_pretty(policy)
        .map_err(|error| format!("failed to serialize cache policy: {error}"))?;
    fs::write(root.join(CACHE_POLICY_FILE), text)
        .map_err(|error| format!("failed to write cache policy: {error}"))
}
