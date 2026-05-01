use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use amigo_modding::ModSceneManifest;
use serde::Deserialize;

use crate::config::{LauncherConfig, LauncherProfile};
use crate::diagnostics::{DiagnosticSeverity, ProfileDiagnostics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Headless,
    Hosted,
}

#[derive(Debug, Clone)]
pub enum TuiOutcome {
    Launch {
        config: LauncherConfig,
        mode: LaunchMode,
    },
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Profiles,
    Tree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TreeEntry {
    Category {
        category_id: String,
    },
    Mod {
        category_id: String,
        mod_index: usize,
    },
    Scene {
        category_id: String,
        mod_index: usize,
        scene_index: usize,
    },
}

#[derive(Debug, Clone)]
struct KnownMod {
    id: String,
    name: String,
    description: String,
    scenes: Vec<ModSceneManifest>,
    launcher_category: Option<Vec<String>>,
    launcher_scene_categories: BTreeMap<String, Vec<String>>,
    discovered: bool,
}

#[derive(Debug, Clone, Default)]
struct LauncherMetadata {
    category: Option<Vec<String>>,
    scene_categories: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LauncherManifestMetadata {
    #[serde(default)]
    launcher_category: Vec<String>,
    #[serde(default)]
    scenes: Vec<LauncherSceneMetadata>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LauncherSceneMetadata {
    #[serde(default)]
    id: String,
    #[serde(default)]
    launcher_category: Vec<String>,
}

#[derive(Debug, Clone)]
struct LauncherTuiState {
    config_path: PathBuf,
    config: LauncherConfig,
    known_mods: Vec<KnownMod>,
    profile_diagnostics: BTreeMap<String, ProfileDiagnostics>,
    resolved_mod_ids: Vec<String>,
    focus: FocusPane,
    selected_profile_index: usize,
    selected_mod_index: usize,
    selected_scene_index: usize,
    tree_cursor_on_scene: bool,
    expanded_mod_ids: BTreeSet<String>,
    scene_filter: String,
    selected_category_id: Option<String>,
    expanded_category_ids: BTreeSet<String>,
    dirty: bool,
    status: String,
}

mod details;
mod discovery;
mod filtering;
mod profiles;
mod render;
mod runtime;
mod state;

pub use runtime::run_launcher_tui;

#[cfg(test)]
mod tests;
