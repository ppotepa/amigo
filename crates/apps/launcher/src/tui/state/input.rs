use super::super::filtering::wrapped_next_index;
use super::super::{FocusPane, LaunchMode, LauncherTuiState, TreeEntry, TuiOutcome};

impl LauncherTuiState {
    pub(crate) fn move_focus_next(&mut self) {
        self.focus = match self.focus {
            FocusPane::Profiles => FocusPane::Tree,
            FocusPane::Tree => FocusPane::Profiles,
        };
        self.status = format!("focus: {}", self.focus_label());
    }

    pub(crate) fn move_focus_previous(&mut self) {
        self.move_focus_next();
    }

    pub(crate) fn focus_label(&self) -> &'static str {
        match self.focus {
            FocusPane::Profiles => "profiles",
            FocusPane::Tree => "tree",
        }
    }

    pub(crate) fn append_scene_filter(&mut self, character: char) {
        self.focus = FocusPane::Tree;
        self.scene_filter.push(character);
        self.sync_tree_selection_to_visible();
        self.status = format!("scene filter: `{}`", self.scene_filter);
    }

    pub(crate) fn pop_scene_filter(&mut self) {
        if self.scene_filter.pop().is_some() {
            self.sync_tree_selection_to_visible();
            self.status = if self.scene_filter.is_empty() {
                "scene filter cleared".to_owned()
            } else {
                format!("scene filter: `{}`", self.scene_filter)
            };
        }
    }

    pub(crate) fn clear_scene_filter(&mut self) {
        self.scene_filter.clear();
        self.sync_tree_selection_to_visible();
        self.status = "scene filter cleared".to_owned();
    }

    pub(crate) fn move_selection(&mut self, delta: isize) {
        match self.focus {
            FocusPane::Profiles => {
                self.selected_profile_index = wrapped_next_index(
                    self.selected_profile_index,
                    self.config.profiles.len(),
                    delta,
                );
            }
            FocusPane::Tree => self.move_tree_selection(delta),
        }
    }

    pub(crate) fn activate_focused(&mut self) -> Option<TuiOutcome> {
        match self.focus {
            FocusPane::Profiles => {
                let Some(profile) = self.config.profiles.get(self.selected_profile_index) else {
                    return None;
                };
                let selected_profile_id = profile.id.clone();
                self.config
                    .set_active_profile(&selected_profile_id)
                    .expect("selected TUI profile should exist");
                self.sync_selection_from_active_profile();
                self.dirty = true;
                self.status = format!("active profile set to `{selected_profile_id}`");
                None
            }
            FocusPane::Tree => {
                if matches!(self.selected_tree_entry(), Some(TreeEntry::Category { .. })) {
                    self.toggle_selected_expansion();
                    None
                } else {
                    self.try_launch_focused(LaunchMode::Hosted)
                }
            }
        }
    }
}
