use std::collections::BTreeSet;
use std::path::PathBuf;

use amigo_core::AmigoResult;
use amigo_modding::{ModSceneManifest, requested_mods_for_root};
use crate::config::{LauncherConfig, LauncherProfile};
use crate::diagnostics::collect_profile_diagnostics;

use super::super::filtering::{
    category_id,
    category_prefixes,
    default_expanded_category_ids,
    launcher_category_for_mod,
    launcher_category_for_scene,
    mod_node_id,
};
use super::super::discovery::discover_known_mods;
use super::super::{FocusPane, LauncherTuiState, ProfileDiagnostics};
use super::super::KnownMod;

impl LauncherTuiState {
    pub(crate) fn new(config_path: PathBuf, config: LauncherConfig) -> AmigoResult<Self> {
        let known_mods = discover_known_mods(&config)?;
        let expanded_category_ids = default_expanded_category_ids(&known_mods);
        let profile_diagnostics = collect_profile_diagnostics(&config);
        let mut state = Self {
            config_path,
            config,
            known_mods,
            profile_diagnostics,
            resolved_mod_ids: Vec::new(),
            focus: FocusPane::Profiles,
            selected_profile_index: 0,
            selected_mod_index: 0,
            selected_scene_index: 0,
            tree_cursor_on_scene: false,
            expanded_mod_ids: BTreeSet::new(),
            scene_filter: String::new(),
            selected_category_id: None,
            expanded_category_ids,
            dirty: false,
            status: "Profiles on top. Type to filter the tree, Enter launches hosted.".to_owned(),
        };
        state.sync_selection_from_active_profile();
        Ok(state)
    }

    pub(crate) fn active_profile(&self) -> &LauncherProfile {
        self.config
            .active_profile()
            .expect("launcher TUI state should always contain a valid active profile")
    }

    pub(crate) fn active_profile_mut(&mut self) -> &mut LauncherProfile {
        self.config
            .active_profile_mut()
            .expect("launcher TUI state should always contain a valid active profile")
    }

    pub(crate) fn active_profile_diagnostics(&self) -> Option<&ProfileDiagnostics> {
        self.profile_diagnostics.get(&self.active_profile().id)
    }

    pub(crate) fn selected_mod(&self) -> Option<&KnownMod> {
        self.known_mods.get(self.selected_mod_index)
    }

    pub(crate) fn selected_scene(&self) -> Option<ModSceneManifest> {
        if self.selected_category_id.is_some() {
            return None;
        }
        if !self.tree_cursor_on_scene {
            return None;
        }

        self.current_scene_list()
            .get(self.selected_scene_index)
            .cloned()
    }

    pub(crate) fn current_scene_list(&self) -> Vec<ModSceneManifest> {
        self.selected_mod()
            .map(|known_mod| known_mod.scenes.clone())
            .unwrap_or_default()
    }

    pub(crate) fn sync_selection_from_active_profile(&mut self) {
        self.scene_filter.clear();
        self.selected_profile_index = self
            .config
            .profiles
            .iter()
            .position(|profile| profile.id == self.config.active_profile)
            .unwrap_or(0);

        let active_profile = self.active_profile();
        self.selected_mod_index = active_profile
            .root_mod
            .as_deref()
            .and_then(|root_mod| {
                self.known_mods
                    .iter()
                    .position(|known_mod| known_mod.id == root_mod)
            })
            .unwrap_or(0);

        self.expanded_mod_ids.clear();
        if let Some(root_mod) = self.active_profile().root_mod.as_deref() {
            self.expanded_mod_ids.insert(root_mod.to_owned());
        }
        self.selected_category_id = None;
        self.expanded_category_ids = default_expanded_category_ids(&self.known_mods);
        self.refresh_resolved_mods();
        self.sync_scene_selection_for_current_mod();
        self.expand_category_for_current_selection();
        self.sync_tree_selection_to_visible();
    }

    pub(crate) fn expand_category_for_current_selection(&mut self) {
        let Some(known_mod) = self.selected_mod() else {
            return;
        };
        let mod_id = known_mod.id.clone();
        let category = if self.tree_cursor_on_scene {
            known_mod
                .scenes
                .get(self.selected_scene_index)
                .map(|scene| launcher_category_for_scene(known_mod, scene))
        } else {
            known_mod
                .scenes
                .first()
                .map(|scene| launcher_category_for_scene(known_mod, scene))
                .or_else(|| Some(launcher_category_for_mod(known_mod)))
        };
        let Some(category) = category else {
            return;
        };
        for prefix in category_prefixes(&category) {
            self.expanded_category_ids.insert(category_id(&prefix));
        }
        self.expanded_mod_ids
            .insert(mod_node_id(&category_id(&category), mod_id.as_str()));
    }

    pub(crate) fn refresh_resolved_mods(&mut self) {
        let root_mod = self.active_profile().root_mod_or_core().to_owned();
        self.resolved_mod_ids = self
            .active_profile_diagnostics()
            .filter(|report| !report.resolved_mod_ids.is_empty())
            .map(|report| report.resolved_mod_ids.clone())
            .unwrap_or_else(|| requested_mods_for_root(&root_mod));
    }

    pub(crate) fn refresh_profile_diagnostics(&mut self) {
        self.profile_diagnostics = collect_profile_diagnostics(&self.config);
        self.refresh_resolved_mods();
    }

    pub(crate) fn sync_scene_selection_for_current_mod(&mut self) {
        let startup_scene = self.active_profile().startup_scene.clone();
        let scenes = self.current_scene_list();

        self.selected_scene_index = startup_scene
            .as_deref()
            .and_then(|scene_id| scenes.iter().position(|scene| scene.id == scene_id))
            .unwrap_or(0);
        self.tree_cursor_on_scene = startup_scene.is_some() && !scenes.is_empty();
    }

}
