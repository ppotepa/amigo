use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_modding::ModCatalog;
use amigo_scene::SceneService;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{queue_scene_reload, queue_scene_select};
use crate::bindings::common::string_array;

#[derive(Clone)]
pub struct SceneApi {
    pub(crate) scene: Option<Arc<SceneService>>,
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) mod_catalog: Option<Arc<ModCatalog>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl SceneApi {
    pub fn current_id(&mut self) -> String {
        active_scene_id(self.scene.as_ref())
    }

    pub fn available(&mut self) -> rhai::Array {
        string_array(available_scene_ids(
            self.launch_selection.as_ref(),
            self.mod_catalog.as_ref(),
        ))
    }

    pub fn has(&mut self, scene_id: &str) -> bool {
        scene_exists_for_selected_mod(
            self.launch_selection.as_ref(),
            self.mod_catalog.as_ref(),
            scene_id,
        )
    }

    pub fn select(&mut self, scene_id: &str) -> bool {
        queue_scene_select(self.command_queue.as_ref(), scene_id)
    }

    pub fn reload(&mut self) {
        queue_scene_reload(self.command_queue.as_ref());
    }
}

pub fn active_scene_id(scene: Option<&Arc<SceneService>>) -> String {
    scene
        .and_then(|scene| scene.selected_scene())
        .map(|scene| scene.as_str().to_owned())
        .unwrap_or_default()
}

pub fn available_scene_ids(
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
        .map(|discovered_mod| {
            discovered_mod
                .manifest
                .scenes
                .iter()
                .map(|scene| scene.id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn scene_exists_for_selected_mod(
    launch_selection: Option<&Arc<LaunchSelection>>,
    mod_catalog: Option<&Arc<ModCatalog>>,
    scene_id: &str,
) -> bool {
    available_scene_ids(launch_selection, mod_catalog)
        .into_iter()
        .any(|known_scene| known_scene == scene_id)
}
