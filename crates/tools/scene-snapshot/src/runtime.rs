use std::path::{Path, PathBuf};

use amigo_app::{capture_scene_preview, ScenePreviewOptions};
use amigo_modding::requested_mods_for_root;

use crate::{
    SceneSnapshotError, SceneSnapshotImage, SceneSnapshotRequest, SceneSnapshotService,
};

#[derive(Clone, Debug)]
pub struct EngineSceneSnapshotService;

impl Default for EngineSceneSnapshotService {
    fn default() -> Self {
        Self
    }
}

pub type RuntimeSceneSnapshotService = EngineSceneSnapshotService;

impl SceneSnapshotService for EngineSceneSnapshotService {
    fn capture(&self, request: SceneSnapshotRequest) -> Result<SceneSnapshotImage, SceneSnapshotError> {
        let mod_root = request
            .mod_root
            .clone()
            .ok_or_else(|| SceneSnapshotError::new("Runtime snapshot request is missing mod_root"))?;
        let mods_root = mods_root_for_mod(&mod_root);

        let options = ScenePreviewOptions::new(
            mods_root,
            request.mod_id.clone(),
            request.scene_id.clone(),
            request.width,
            request.height,
        )
        .with_active_mods(requested_mods_for_root(&request.mod_id))
        .with_warmup_frames(request.warmup_frames());

        let frame = capture_scene_preview(options).map_err(|err| {
            SceneSnapshotError::new(format!(
                "Engine preview snapshot failed for `{}` / `{}`: {err}",
                request.mod_id, request.scene_id
            ))
        })?;

        Ok(SceneSnapshotImage {
            request,
            width: frame.width,
            height: frame.height,
            pixels_rgba8: frame.pixels_rgba8,
            diagnostic_label: frame.diagnostic_label,
        })
    }
}

fn mods_root_for_mod(mod_root: &Path) -> PathBuf {
    mod_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("mods"))
}
