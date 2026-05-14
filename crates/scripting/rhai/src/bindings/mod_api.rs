use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_modding::ModCatalog;

use crate::bindings::common::string_array;
use crate::bindings::scene::{available_scene_ids, scene_exists_for_selected_mod};

#[derive(Clone)]
pub struct ModApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) mod_catalog: Option<Arc<ModCatalog>>,
}

impl ModApi {
    pub fn current_id(&mut self) -> String {
        selected_mod_id(self.launch_selection.as_ref())
    }

    pub fn scenes(&mut self) -> rhai::Array {
        string_array(available_scene_ids(
            self.launch_selection.as_ref(),
            self.mod_catalog.as_ref(),
        ))
    }

    pub fn has_scene(&mut self, scene_id: &str) -> bool {
        scene_exists_for_selected_mod(
            self.launch_selection.as_ref(),
            self.mod_catalog.as_ref(),
            scene_id,
        )
    }

    pub fn capabilities(&mut self) -> rhai::Array {
        string_array(capabilities_for_selected_mod(
            self.launch_selection.as_ref(),
            self.mod_catalog.as_ref(),
        ))
    }

    pub fn loaded(&mut self) -> rhai::Array {
        string_array(loaded_mod_ids(self.mod_catalog.as_ref()))
    }
}

pub fn selected_mod_id(launch_selection: Option<&Arc<LaunchSelection>>) -> String {
    launch_selection
        .map(|selection| selection.selected_mod().to_owned())
        .unwrap_or_default()
}

pub fn capabilities_for_selected_mod(
    launch_selection: Option<&Arc<LaunchSelection>>,
    mod_catalog: Option<&Arc<ModCatalog>>,
) -> Vec<String> {
    let Some(selected_mod) = launch_selection.map(|selection| selection.selected_mod()) else {
        return Vec::new();
    };
    let Some(mod_catalog) = mod_catalog else {
        return Vec::new();
    };

    mod_catalog
        .mod_by_id(selected_mod)
        .map(|discovered_mod| discovered_mod.manifest.capabilities.clone())
        .unwrap_or_default()
}

pub fn loaded_mod_ids(mod_catalog: Option<&Arc<ModCatalog>>) -> Vec<String> {
    mod_catalog
        .map(|mod_catalog| {
            mod_catalog
                .mod_ids()
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
