use super::super::filtering::mod_node_id;
use super::super::TreeEntry;
use super::super::LauncherTuiState;

impl LauncherTuiState {
    pub(crate) fn toggle_selected_expansion(&mut self) {
        let Some(entry) = self.selected_tree_entry() else {
            return;
        };
        match entry {
            TreeEntry::Category { category_id } => {
                if self.expanded_category_ids.contains(&category_id) {
                    self.expanded_category_ids.remove(&category_id);
                    self.status = format!("collapsed `{category_id}`");
                } else {
                    self.expanded_category_ids.insert(category_id.clone());
                    self.status = format!("expanded `{category_id}`");
                }
            }
            TreeEntry::Mod {
                category_id,
                mod_index,
            }
            | TreeEntry::Scene {
                category_id,
                mod_index,
                ..
            } => {
                let mod_id = self.known_mods[mod_index].id.clone();
                let node_id = mod_node_id(&category_id, &mod_id);
                if self.expanded_mod_ids.contains(&node_id)
                    || self.expanded_mod_ids.contains(&mod_id)
                {
                    self.expanded_mod_ids.remove(&node_id);
                    self.expanded_mod_ids.remove(&mod_id);
                    if self.tree_cursor_on_scene {
                        self.tree_cursor_on_scene = false;
                    }
                    self.status = format!("collapsed `{mod_id}`");
                } else {
                    self.expanded_mod_ids.insert(node_id);
                    self.status = format!("expanded `{mod_id}`");
                }
            }
        }
        self.sync_tree_selection_to_visible();
    }

    pub(crate) fn expand_selected_mod(&mut self) {
        let Some(entry) = self.selected_tree_entry() else {
            return;
        };
        match entry {
            TreeEntry::Category { category_id } => {
                if self.expanded_category_ids.insert(category_id.clone()) {
                    self.status = format!("expanded `{category_id}`");
                }
            }
            TreeEntry::Mod {
                category_id,
                mod_index,
            } => {
                let mod_id = self.known_mods[mod_index].id.clone();
                if self
                    .expanded_mod_ids
                    .insert(mod_node_id(&category_id, &mod_id))
                {
                    self.status = format!("expanded `{mod_id}`");
                }
            }
            TreeEntry::Scene { .. } => {}
        }
        self.sync_tree_selection_to_visible();
    }

    pub(crate) fn collapse_selected_mod_or_parent(&mut self) {
        if self.tree_cursor_on_scene {
            self.tree_cursor_on_scene = false;
            self.status = "moved to parent mod".to_owned();
            return;
        }

        let Some(entry) = self.selected_tree_entry() else {
            return;
        };
        match entry {
            TreeEntry::Category { category_id } => {
                if self.expanded_category_ids.remove(&category_id) {
                    self.status = format!("collapsed `{category_id}`");
                }
            }
            TreeEntry::Mod {
                category_id,
                mod_index,
            } => {
                let mod_id = self.known_mods[mod_index].id.clone();
                if self
                    .expanded_mod_ids
                    .remove(&mod_node_id(&category_id, &mod_id))
                    || self.expanded_mod_ids.remove(&mod_id)
                {
                    self.status = format!("collapsed `{mod_id}`");
                }
            }
            TreeEntry::Scene { .. } => {}
        }
        self.sync_tree_selection_to_visible();
    }
}
