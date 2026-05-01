use super::super::filtering::{
    category_id,
    category_prefixes,
    category_matches_filter,
    compare_launcher_category_paths,
    launcher_category_for_mod,
    launcher_category_for_scene,
    mod_matches_filter,
    mod_node_id,
    scene_matches_filter,
    wrapped_next_index,
};
use super::super::{KnownMod, TreeEntry, LauncherTuiState};
use std::collections::{BTreeMap, BTreeSet};

impl LauncherTuiState {
    pub(crate) fn find_mod(&self, mod_id: &str) -> Option<&KnownMod> {
        self.known_mods
            .iter()
            .find(|known_mod| known_mod.id == mod_id)
    }

    pub(crate) fn selected_tree_entry(&self) -> Option<TreeEntry> {
        let entries = self.visible_tree_entries();
        if let Some(category_id) = self.selected_category_id.as_ref() {
            return entries.into_iter().find(|entry| {
                matches!(entry, TreeEntry::Category { category_id: id } if id == category_id)
            });
        }

        entries.into_iter().find(|entry| match entry {
            TreeEntry::Scene {
                mod_index,
                scene_index,
                ..
            } => {
                self.tree_cursor_on_scene
                    && *mod_index == self.selected_mod_index
                    && *scene_index == self.selected_scene_index
            }
            TreeEntry::Mod { mod_index, .. } => {
                !self.tree_cursor_on_scene && *mod_index == self.selected_mod_index
            }
            TreeEntry::Category { .. } => false,
        })
    }

    pub(crate) fn visible_tree_entries(&self) -> Vec<TreeEntry> {
        let filter_active = !self.scene_filter.trim().is_empty();
        let mut grouped: BTreeMap<Vec<String>, BTreeMap<usize, BTreeSet<usize>>> = BTreeMap::new();
        for (mod_index, known_mod) in self.known_mods.iter().enumerate() {
            let mod_matches = filter_active && mod_matches_filter(known_mod, &self.scene_filter);
            if known_mod.scenes.is_empty() {
                let category = launcher_category_for_mod(known_mod);
                let category_matches =
                    filter_active && category_matches_filter(&category, &self.scene_filter);
                if !filter_active || mod_matches || category_matches {
                    grouped
                        .entry(category)
                        .or_default()
                        .entry(mod_index)
                        .or_default();
                }
                continue;
            }

            for (scene_index, scene) in known_mod.scenes.iter().enumerate() {
                let category = launcher_category_for_scene(known_mod, scene);
                let category_matches =
                    filter_active && category_matches_filter(&category, &self.scene_filter);
                let scene_matches =
                    filter_active && scene_matches_filter(scene, &self.scene_filter);
                if !filter_active || mod_matches || scene_matches || category_matches {
                    grouped
                        .entry(category)
                        .or_default()
                        .entry(mod_index)
                        .or_default()
                        .insert(scene_index);
                }
            }
        }

        let mut entries = Vec::new();
        let mut pushed_categories = BTreeSet::new();
        let mut categories = grouped.keys().cloned().collect::<Vec<_>>();
        categories.sort_by(compare_launcher_category_paths);

        for category in categories {
            let mut ancestors_expanded = true;
            for prefix in category_prefixes(&category) {
                let id = category_id(&prefix);
                if pushed_categories.insert(id.clone()) {
                    entries.push(TreeEntry::Category {
                        category_id: id.clone(),
                    });
                }
                if !filter_active && !self.expanded_category_ids.contains(&id) {
                    ancestors_expanded = false;
                    break;
                }
            }
            if !ancestors_expanded {
                continue;
            }

            let category_id = category_id(&category);
            let Some(mods) = grouped.get(&category) else {
                continue;
            };
            for (mod_index, scene_indices) in mods {
                entries.push(TreeEntry::Mod {
                    category_id: category_id.clone(),
                    mod_index: *mod_index,
                });

                let known_mod = &self.known_mods[*mod_index];
                let mod_expanded = filter_active
                    || self
                        .expanded_mod_ids
                        .contains(&mod_node_id(&category_id, known_mod.id.as_str()))
                    || self.expanded_mod_ids.contains(&known_mod.id);
                if !mod_expanded {
                    continue;
                }

                for scene_index in scene_indices.iter().copied() {
                    entries.push(TreeEntry::Scene {
                        category_id: category_id.clone(),
                        mod_index: *mod_index,
                        scene_index,
                    });
                }
            }
        }

        entries
    }

    pub(crate) fn sync_tree_selection_to_visible(&mut self) {
        let entries = self.visible_tree_entries();
        if entries.is_empty() {
            self.selected_mod_index = 0;
            self.selected_scene_index = 0;
            self.tree_cursor_on_scene = false;
            return;
        }

        if let Some(selected) = self.selected_tree_entry() {
            if entries.iter().any(|entry| *entry == selected)
                && (!self.filter_prefers_scene_selection()
                    || matches!(selected, TreeEntry::Scene { .. }))
            {
                return;
            }
        }

        if let Some(entry) = self.preferred_filtered_scene_entry(&entries) {
            self.apply_tree_entry(entry);
            return;
        }

        self.apply_tree_entry(entries[0].clone());
    }

    pub(crate) fn apply_tree_entry(&mut self, entry: TreeEntry) {
        match entry {
            TreeEntry::Category { category_id } => {
                self.selected_category_id = Some(category_id);
                self.tree_cursor_on_scene = false;
            }
            TreeEntry::Mod { mod_index, .. } => {
                self.selected_category_id = None;
                self.selected_mod_index = mod_index;
                self.tree_cursor_on_scene = false;
            }
            TreeEntry::Scene {
                mod_index,
                scene_index,
                ..
            } => {
                self.selected_category_id = None;
                self.selected_mod_index = mod_index;
                self.selected_scene_index = scene_index;
                self.tree_cursor_on_scene = true;
            }
        }
    }

    pub(crate) fn move_tree_selection(&mut self, delta: isize) {
        let entries = self.visible_tree_entries();
        if entries.is_empty() {
            return;
        }

        let current = self
            .selected_tree_entry()
            .and_then(|selected| entries.iter().position(|entry| *entry == selected))
            .unwrap_or(0);
        let next = wrapped_next_index(current, entries.len(), delta);
        self.apply_tree_entry(entries[next].clone());
    }

    pub(crate) fn filter_prefers_scene_selection(&self) -> bool {
        !self.scene_filter.trim().is_empty() && self.first_matching_scene_entry().is_some()
    }

    pub(crate) fn preferred_filtered_scene_entry(&self, entries: &[TreeEntry]) -> Option<TreeEntry> {
        if !self.filter_prefers_scene_selection() {
            return None;
        }

        self.first_matching_scene_entry()
            .filter(|entry| entries.contains(entry))
    }

    pub(crate) fn first_matching_scene_entry(&self) -> Option<TreeEntry> {
        if self.scene_filter.trim().is_empty() {
            return None;
        }

        for (mod_index, known_mod) in self.known_mods.iter().enumerate() {
            if !mod_matches_filter(known_mod, &self.scene_filter) {
                for (scene_index, scene) in known_mod.scenes.iter().enumerate() {
                    if scene_matches_filter(scene, &self.scene_filter) {
                        return Some(TreeEntry::Scene {
                            category_id: category_id(&launcher_category_for_scene(
                                known_mod, scene,
                            )),
                            mod_index,
                            scene_index,
                        });
                    }
                }
                continue;
            }

            if let Some((scene_index, scene)) = known_mod
                .scenes
                .iter()
                .enumerate()
                .find(|(_, scene)| scene_matches_filter(scene, &self.scene_filter))
            {
                return Some(TreeEntry::Scene {
                    category_id: category_id(&launcher_category_for_scene(known_mod, scene)),
                    mod_index,
                    scene_index,
                });
            }
        }

        None
    }

}
