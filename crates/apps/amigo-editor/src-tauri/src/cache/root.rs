use std::env;
use std::fs;
use std::path::PathBuf;

use crate::dto::EditorSettingsDto;

const SETTINGS_FILE: &str = "target/amigo-editor/editor-settings.json";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CacheRootMode {
    WorkspaceTarget,
    EnvOverride,
    AppCache,
    Custom(PathBuf),
}

#[derive(Debug, Clone)]
pub struct CacheRoot {
    pub path: PathBuf,
    pub mode: CacheRootMode,
}

#[derive(Debug, Clone)]
pub struct EditorPaths {
    pub cache_root: PathBuf,
    pub cache_root_mode: String,
}

#[allow(dead_code)]
pub fn resolve_cache_root() -> CacheRoot {
    resolve_cache_root_with_app_cache(None)
}

pub fn resolve_cache_root_with_app_cache(app_cache_root: Option<PathBuf>) -> CacheRoot {
    if let Ok(path) = env::var("AMIGO_EDITOR_CACHE_DIR") {
        return CacheRoot {
            path: PathBuf::from(path),
            mode: CacheRootMode::EnvOverride,
        };
    }

    if let Some(path) = settings_cache_root_override() {
        return CacheRoot {
            path: PathBuf::from(&path),
            mode: CacheRootMode::Custom(PathBuf::from(path)),
        };
    }

    if let Some(path) = app_cache_root {
        return CacheRoot {
            path,
            mode: CacheRootMode::AppCache,
        };
    }

    CacheRoot {
        path: PathBuf::from("target").join("amigo-editor").join("cache"),
        mode: CacheRootMode::WorkspaceTarget,
    }
}

pub fn cache_root_mode_name(mode: &CacheRootMode) -> String {
    match mode {
        CacheRootMode::WorkspaceTarget => "WorkspaceTarget".to_owned(),
        CacheRootMode::EnvOverride => "EnvOverride".to_owned(),
        CacheRootMode::AppCache => "AppCache".to_owned(),
        CacheRootMode::Custom(_) => "Custom".to_owned(),
    }
}

fn settings_cache_root_override() -> Option<String> {
    let text = fs::read_to_string(SETTINGS_FILE).ok()?;
    let settings: EditorSettingsDto = serde_json::from_str(&text).ok()?;
    settings.cache_root_override
}
