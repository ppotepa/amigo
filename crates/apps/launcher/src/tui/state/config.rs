use amigo_core::AmigoResult;
use crate::config::LauncherConfig;

use super::super::discovery::discover_known_mods;
use super::super::{
    DiagnosticSeverity, FocusPane, LaunchMode, LauncherTuiState, TuiOutcome, TreeEntry,
};

impl LauncherTuiState {
    pub(crate) fn toggle_hosted_default(&mut self) {
        let (profile_id, hosted_default) = {
            let profile = self.active_profile_mut();
            profile.hosted_default = !profile.hosted_default;
            (profile.id.clone(), profile.hosted_default)
        };
        self.refresh_profile_diagnostics();
        self.dirty = true;
        self.status = format!(
            "profile `{}` hosted_default set to {}",
            profile_id, hosted_default
        );
    }

    pub(crate) fn set_root_mod(&mut self, mod_id: &str) {
        let mod_scenes = self
            .find_mod(mod_id)
            .map(|known_mod| known_mod.scenes.clone())
            .unwrap_or_default();
        self.scene_filter.clear();
        self.expanded_mod_ids.insert(mod_id.to_owned());
        let profile = self.active_profile_mut();
        profile.root_mod = Some(mod_id.to_owned());
        profile.startup_scene = mod_scenes.first().map(|scene| scene.id.clone());

        self.refresh_profile_diagnostics();
        self.dirty = true;
        self.status = format!(
            "root mod set to `{mod_id}` with startup scene `{}`",
            self.active_profile()
                .startup_scene
                .as_deref()
                .unwrap_or("none")
        );
        self.sync_selection_from_active_profile();
    }

    pub(crate) fn set_startup_scene(&mut self, scene_id: &str) {
        let profile = self.active_profile_mut();
        profile.startup_scene = Some(scene_id.to_owned());
        self.refresh_profile_diagnostics();
        self.dirty = true;
        self.status = format!("startup scene set to `{scene_id}`");
        self.sync_scene_selection_for_current_mod();
    }

    pub(crate) fn save_config(&mut self) -> AmigoResult<()> {
        self.config.save(&self.config_path)?;
        self.dirty = false;
        self.status = format!("saved launcher config to `{}`", self.config_path.display());
        Ok(())
    }

    pub(crate) fn reload_config(&mut self) -> AmigoResult<()> {
        let config = LauncherConfig::load(&self.config_path)?;
        config.validate_phase1()?;
        self.known_mods = discover_known_mods(&config)?;
        self.config = config;
        self.scene_filter.clear();
        self.refresh_profile_diagnostics();
        self.dirty = false;
        self.sync_selection_from_active_profile();
        self.status = format!(
            "reloaded launcher config from `{}`",
            self.config_path.display()
        );
        Ok(())
    }

    pub(crate) fn try_launch(&mut self, mode: LaunchMode) -> Option<TuiOutcome> {
        let Some(report) = self.active_profile_diagnostics().cloned() else {
            let profile_id = self.active_profile().id.clone();
            self.status = format!("profile `{profile_id}` has no diagnostics report");
            return None;
        };
        let profile_id = report.profile_id.clone();

        if !report.is_launchable() {
            let first_error = report
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
                .map(|diagnostic| diagnostic.message.clone())
                .unwrap_or_else(|| "unknown launch error".to_owned());
            self.status = format!("profile `{profile_id}` is blocked: {first_error}");
            return None;
        }

        if report.warning_count() > 0 {
            self.status = format!(
                "launching profile `{profile_id}` with {} warning(s)",
                report.warning_count()
            );
        } else {
            self.status = format!("launching profile `{profile_id}`");
        }

        Some(TuiOutcome::Launch {
            config: self.config.clone(),
            mode,
        })
    }

    pub(crate) fn sync_profile_to_focused_selection(&mut self) {
        match self.focus {
            FocusPane::Profiles => {}
            FocusPane::Tree => {
                if matches!(self.selected_tree_entry(), Some(TreeEntry::Category { .. })) {
                    return;
                }
                let selected_mod_id = self.selected_mod().map(|known_mod| known_mod.id.clone());
                let selected_scene_id = self.selected_scene().map(|scene| scene.id);

                if let Some(mod_id) = selected_mod_id {
                    if self.active_profile().root_mod.as_deref() != Some(mod_id.as_str()) {
                        self.set_root_mod(&mod_id);
                    }
                    if let Some(scene_id) = selected_scene_id {
                        self.set_startup_scene(&scene_id);
                    }
                }
            }
        }
    }

    pub(crate) fn try_launch_focused(&mut self, mode: LaunchMode) -> Option<TuiOutcome> {
        self.sync_profile_to_focused_selection();
        self.try_launch(mode)
    }
}
