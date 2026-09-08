use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

use amigo_core::{AmigoError, AmigoResult};
use amigo_modding::ModCatalog;
use amigo_runtime::Runtime;
use amigo_session::SceneSessionService;

use crate::{
    AuthoringSceneGraph, AuthoringSourceScalarPatch, load_authoring_scene_graph,
    patch_yaml_source_scalar, write_yaml_source_atomically,
};

#[derive(Debug, Default)]
pub struct AuthoringSceneGraphService {
    cache: Mutex<BTreeMap<SceneCacheKey, CachedGraph>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SceneCacheKey {
    source_mod: String,
    scene_id: String,
}

#[derive(Debug, Clone)]
struct CachedGraph {
    graph: AuthoringSceneGraph,
    stamps: BTreeMap<PathBuf, FileStamp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    modified_millis: Option<u128>,
    byte_len: u64,
    exists: bool,
}

impl AuthoringSceneGraphService {
    pub fn graph_for_current_scene(&self, runtime: &Runtime) -> AmigoResult<AuthoringSceneGraph> {
        let (source_mod, scene_id) = current_scene_context(runtime)?;
        let key = SceneCacheKey {
            source_mod,
            scene_id,
        };

        let mut cache = self
            .cache
            .lock()
            .expect("authoring scene graph cache mutex should not be poisoned");

        if let Some(cached) = cache.get(&key) {
            if !cached.is_stale() {
                return Ok(cached.graph.clone());
            }
        }

        let graph = load_authoring_scene_graph(runtime)?;
        let stamps = stamps_for_files(&graph.source_files);
        cache.insert(
            key,
            CachedGraph {
                graph: graph.clone(),
                stamps,
            },
        );
        Ok(graph)
    }

    pub fn invalidate_scene(&self, source_mod: &str, scene_id: &str) {
        self.cache
            .lock()
            .expect("authoring scene graph cache mutex should not be poisoned")
            .remove(&SceneCacheKey {
                source_mod: source_mod.to_owned(),
                scene_id: scene_id.to_owned(),
            });
    }

    /// Persists one conservative scalar edit in a source file owned by the
    /// current scene. The patcher preserves surrounding YAML text and rejects
    /// stale or ambiguous changes before any file is replaced.
    pub fn apply_source_scalar_patch(
        &self,
        runtime: &Runtime,
        patch: AuthoringSourceScalarPatch,
    ) -> AmigoResult<()> {
        let (source_mod, scene_id) = current_scene_context(runtime)?;
        let graph = self.graph_for_current_scene(runtime)?;
        let requested = patch.source_file.canonicalize().map_err(|error| {
            AmigoError::Message(format!(
                "editor authoring: source patch file is unavailable: {error}"
            ))
        })?;
        let owned = graph.source_files.iter().any(|file| {
            file.canonicalize()
                .is_ok_and(|candidate| candidate == requested)
        });
        if !owned {
            return Err(AmigoError::Message(format!(
                "editor authoring: source patch target is not owned by current scene: {}",
                requested.display()
            )));
        }
        let source = std::fs::read_to_string(&requested).map_err(|error| {
            AmigoError::Message(format!("editor authoring: cannot read source patch: {error}"))
        })?;
        let output = patch_yaml_source_scalar(
            &source,
            &patch.yaml_pointer,
            &patch.expected,
            &patch.replacement,
        )
        .map_err(|error| AmigoError::Message(format!("editor authoring: {error}")))?;
        write_yaml_source_atomically(&requested, &output)
            .map_err(|error| AmigoError::Message(format!("editor authoring: {error}")))?;
        self.invalidate_scene(&source_mod, &scene_id);
        Ok(())
    }

    pub fn invalidate_all(&self) {
        self.cache
            .lock()
            .expect("authoring scene graph cache mutex should not be poisoned")
            .clear();
    }

    pub fn cached_scene_count(&self) -> usize {
        self.cache
            .lock()
            .expect("authoring scene graph cache mutex should not be poisoned")
            .len()
    }
}

fn current_scene_context(runtime: &Runtime) -> AmigoResult<(String, String)> {
    let scene_session = runtime.required::<SceneSessionService>()?;
    let scene_snapshot = scene_session.snapshot();
    let loaded = scene_snapshot.loaded_scene_document().ok_or_else(|| {
        amigo_core::AmigoError::Message("editor authoring: no loaded scene document".to_owned())
    })?;

    // Require ModCatalog here so cache lookup has the same service preconditions
    // as graph loading. This keeps missing modding setup reported early.
    runtime.required::<ModCatalog>()?;
    Ok((loaded.source_mod.clone(), loaded.scene_id.clone()))
}

impl CachedGraph {
    fn is_stale(&self) -> bool {
        self.stamps
            .iter()
            .any(|(path, stamp)| file_stamp(path) != *stamp)
    }
}

fn stamps_for_files(paths: &[PathBuf]) -> BTreeMap<PathBuf, FileStamp> {
    paths
        .iter()
        .map(|path| (path.clone(), file_stamp(path)))
        .collect()
}

fn file_stamp(path: &Path) -> FileStamp {
    let Ok(metadata) = std::fs::metadata(path) else {
        return FileStamp {
            modified_millis: None,
            byte_len: 0,
            exists: false,
        };
    };

    let modified_millis = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis());

    FileStamp {
        modified_millis,
        byte_len: metadata.len(),
        exists: true,
    }
}
