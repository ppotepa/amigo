use amigo_core::{AmigoError, AmigoResult, LaunchSelection};
use amigo_modding::ModCatalog;
use amigo_runtime::{Runtime, RuntimePlugin, ServiceRegistry};

use crate::runtime_context::required;
use crate::BootstrapOptions;

pub(crate) fn build_launch_selection(options: &BootstrapOptions) -> LaunchSelection {
    LaunchSelection::new(
        options.startup_mod.clone(),
        options.startup_scene.clone(),
        options.active_mods.clone().unwrap_or_default(),
        options.dev_mode,
    )
}

pub(crate) fn validate_launch_selection(
    runtime: &Runtime,
    launch_selection: &LaunchSelection,
) -> AmigoResult<()> {
    let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
        return Ok(());
    };
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let discovered_mod = mod_catalog.mod_by_id(startup_mod).ok_or_else(|| {
        AmigoError::Message(format!(
            "root mod `{startup_mod}` was not loaded by the current bootstrap selection"
        ))
    })?;

    if let Some(startup_scene) = launch_selection.startup_scene.as_deref() {
        if discovered_mod.scene_by_id(startup_scene).is_none() {
            return Err(AmigoError::Message(format!(
                "startup scene `{startup_scene}` was not declared by root mod `{startup_mod}`"
            )));
        }
    }

    Ok(())
}

pub(crate) fn scene_ids_for_launch_selection(
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
) -> Vec<String> {
    launch_selection
        .startup_mod
        .as_deref()
        .and_then(|root_mod| mod_catalog.mod_by_id(root_mod))
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

pub(crate) fn next_scene_id(
    scene_ids: &[String],
    active_scene: Option<&str>,
    step: isize,
) -> Option<String> {
    if scene_ids.is_empty() {
        return None;
    }

    let current_index = active_scene
        .and_then(|active_scene| {
            scene_ids
                .iter()
                .position(|scene_id| scene_id == active_scene)
        })
        .unwrap_or(0);
    let len = scene_ids.len() as isize;
    let next_index = (current_index as isize + step).rem_euclid(len) as usize;

    scene_ids.get(next_index).cloned()
}

pub(crate) struct LaunchSelectionPlugin {
    selection: LaunchSelection,
}

impl LaunchSelectionPlugin {
    pub(crate) fn new(selection: LaunchSelection) -> Self {
        Self { selection }
    }
}

impl RuntimePlugin for LaunchSelectionPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-launch-selection"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(self.selection.clone())
    }
}
