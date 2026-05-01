use std::fs;
use std::path::{Path, PathBuf};

use super::core::SceneDocument;
use crate::{SceneDocumentError, SceneDocumentResult};

pub fn load_scene_document_from_str(source: &str) -> SceneDocumentResult<SceneDocument> {
    serde_yaml::from_str::<SceneDocument>(source)
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

pub fn load_scene_document_from_path(path: impl AsRef<Path>) -> SceneDocumentResult<SceneDocument> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|source| SceneDocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    serde_yaml::from_str::<SceneDocument>(&raw).map_err(|source| SceneDocumentError::Parse {
        path: Some(path.to_path_buf()),
        source,
    })
}

pub fn scene_document_path(
    mod_root: impl AsRef<Path>,
    relative_document_path: impl AsRef<Path>,
) -> PathBuf {
    mod_root.as_ref().join(relative_document_path.as_ref())
}

