use std::sync::Arc;

use amigo_assets::{
    AssetCatalog, discover_glb_mesh3d_assets, discovered_mesh3d_assets,
};
use amigo_core::LaunchSelection;
use amigo_modding::ModCatalog;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::common::string_array;
use crate::bindings::commands::{
    queue_mesh3d_apply_npr_preset, queue_mesh3d_set_npr_gpu_debug_mode,
    queue_mesh3d_set_npr_render_strategy, queue_mesh3d_set_npr_temporal_path_smoothing,
    queue_mesh3d_set_mesh_asset, queue_mesh3d_spawn,
};

#[derive(Clone)]
pub struct Mesh3dApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) asset_catalog: Option<Arc<AssetCatalog>>,
    pub(crate) mod_catalog: Option<Arc<ModCatalog>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Mesh3dApi {
    pub fn queue(&mut self, entity_name: &str, mesh_key: &str) -> bool {
        queue_mesh3d_spawn(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            mesh_key,
        )
    }

    pub fn apply_npr_preset(&mut self, entity_name: &str, preset_id: &str) -> bool {
        queue_mesh3d_apply_npr_preset(self.command_queue.as_ref(), entity_name, preset_id)
    }

    pub fn set_npr_temporal_path_smoothing(&mut self, entity_name: &str, enabled: bool) -> bool {
        queue_mesh3d_set_npr_temporal_path_smoothing(
            self.command_queue.as_ref(),
            entity_name,
            enabled,
        )
    }

    pub fn set_npr_render_strategy(&mut self, entity_name: &str, strategy: &str) -> bool {
        queue_mesh3d_set_npr_render_strategy(self.command_queue.as_ref(), entity_name, strategy)
    }

    pub fn set_npr_gpu_debug_mode(&mut self, entity_name: &str, debug_mode: &str) -> bool {
        queue_mesh3d_set_npr_gpu_debug_mode(self.command_queue.as_ref(), entity_name, debug_mode)
    }

    pub fn set_mesh(&mut self, entity_name: &str, mesh_key: &str) -> bool {
        queue_mesh3d_set_mesh_asset(self.command_queue.as_ref(), entity_name, mesh_key)
    }

    pub fn set_mesh_asset(&mut self, entity_name: &str, mesh_key: &str) -> bool {
        self.set_mesh(entity_name, mesh_key)
    }

    pub fn scan_models(&mut self) -> rhai::INT {
        let mod_id = self
            .launch_selection
            .as_ref()
            .map(|selection| selection.selected_mod().to_owned())
            .unwrap_or_default();
        self.scan_models_for_mod(mod_id.as_str())
    }

    pub fn scan_models_for_mod(&mut self, mod_id: &str) -> rhai::INT {
        let Some(asset_catalog) = self.asset_catalog.as_ref() else {
            return 0;
        };
        let Some(mod_catalog) = self.mod_catalog.as_ref() else {
            return 0;
        };
        let Some(discovered_mod) = mod_catalog.mod_by_id(mod_id) else {
            return 0;
        };
        discover_glb_mesh3d_assets(asset_catalog, &discovered_mod.root_path, mod_id)
            .map(|models| models.len() as rhai::INT)
            .unwrap_or(0)
    }

    pub fn models(&mut self) -> rhai::Array {
        string_array(
            self.asset_catalog
                .as_ref()
                .map(|catalog| {
                    discovered_mesh3d_assets(catalog)
                        .into_iter()
                        .map(|model| model.key.as_str().to_owned())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        )
    }

    pub fn model_count(&mut self) -> rhai::INT {
        self.asset_catalog
            .as_ref()
            .map(|catalog| discovered_mesh3d_assets(catalog).len() as rhai::INT)
            .unwrap_or(0)
    }

    pub fn model_asset(&mut self, index: rhai::INT) -> String {
        let Some(asset_catalog) = self.asset_catalog.as_ref() else {
            return String::new();
        };
        let Ok(index) = usize::try_from(index) else {
            return String::new();
        };
        discovered_mesh3d_assets(asset_catalog)
            .get(index)
            .map(|model| model.key.as_str().to_owned())
            .unwrap_or_default()
    }

    pub fn model_label(&mut self, index: rhai::INT) -> String {
        let Some(asset_catalog) = self.asset_catalog.as_ref() else {
            return String::new();
        };
        let Ok(index) = usize::try_from(index) else {
            return String::new();
        };
        discovered_mesh3d_assets(asset_catalog)
            .get(index)
            .map(|model| model.label.clone())
            .unwrap_or_default()
    }

    pub fn model_index_by_label(&mut self, fragment: &str) -> rhai::INT {
        let Some(asset_catalog) = self.asset_catalog.as_ref() else {
            return -1;
        };
        let fragment = fragment.to_ascii_lowercase();
        discovered_mesh3d_assets(asset_catalog)
            .iter()
            .position(|model| model.label.to_ascii_lowercase().contains(&fragment))
            .map(|index| index as rhai::INT)
            .unwrap_or(-1)
    }
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<Mesh3dApi>("WorldMesh3d")
        .register_fn("queue", Mesh3dApi::queue)
        .register_fn("apply_npr_preset", Mesh3dApi::apply_npr_preset)
        .register_fn(
            "set_npr_temporal_path_smoothing",
            Mesh3dApi::set_npr_temporal_path_smoothing,
        )
        .register_fn("set_npr_render_strategy", Mesh3dApi::set_npr_render_strategy)
        .register_fn("set_npr_gpu_debug_mode", Mesh3dApi::set_npr_gpu_debug_mode)
        .register_fn("set_mesh", Mesh3dApi::set_mesh)
        .register_fn("set_mesh_asset", Mesh3dApi::set_mesh_asset)
        .register_fn("scan_models", Mesh3dApi::scan_models)
        .register_fn("scan_models", Mesh3dApi::scan_models_for_mod)
        .register_fn("models", Mesh3dApi::models)
        .register_fn("model_count", Mesh3dApi::model_count)
        .register_fn("model_asset", Mesh3dApi::model_asset)
        .register_fn("model_label", Mesh3dApi::model_label)
        .register_fn("model_index_by_label", Mesh3dApi::model_index_by_label);
}
