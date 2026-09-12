//! Shared, typed domain endpoint for companion and scripting adapters.
use crate::{
    NprPlaygroundState,
    documents::*,
    state::{MODELS, ObjectSettings, Settings},
};
use amigo_playground_api::*;
use amigo_render_npr::{ComicInk, NprStyleLayers};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub type NprPlaygroundActionError = PlaygroundActionError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NprPlaygroundIntent {
    BeginCameraGesture {
        gesture_id: u64,
    },
    EndCameraGesture {
        gesture_id: u64,
    },
    SetSeed {
        seed: u64,
    },
    RefreshCatalog {
        catalog: String,
    },
    ImportRemote {
        catalog: String,
        model: String,
    },
    ImportModel {
        path: PathBuf,
    },
    Navigate {
        mode: String,
        dx: f32,
        dy: f32,
        wheel: f32,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        focused: bool,
    },
    Select {
        object: String,
    },
    SetCamera {
        target: glam::Vec3,
        yaw: f32,
        pitch: f32,
        distance: f32,
        fov: f32,
    },
    SetMotion {
        paused: bool,
        speed: f32,
        sketch_paused: bool,
    },
    SetTemporalPolicy {
        policy: amigo_render_npr::NprMotionPolicy,
    },
    SetDebugView {
        view: String,
    },
    SetLayerLock {
        layer: String,
        locked: bool,
    },
    UseLook {
        id: String,
    },
    SetObject {
        object: String,
        settings: ObjectSettings,
    },
    SetObjectPose {
        object: String,
        position: glam::Vec3,
        rotation: glam::Vec3,
        scale: f32,
    },
    SetLook {
        style: ComicInk,
        layers: NprStyleLayers,
    },
    SelectModel {
        model: String,
    },
    OpenDraft {
        model: String,
    },
    RemoveObject {
        object: String,
    },
    Undo,
    Redo,
    SaveAll,
    SaveLook,
    SaveAsLook {
        id: String,
    },
    SaveVariant {
        id: String,
    },
    ApplyVariant {
        id: String,
    },
    UpdateBrushVersion {
        id: String,
        from: u32,
        to: u32,
    },
    SetBuildUp {
        value: f32,
    },
    SetSoloLayer {
        layer: Option<String>,
    },
    SaveBrushVersion {
        brush: amigo_render_npr::BrushDefinition,
    },
    Reload,
}

#[derive(Debug, Clone, Serialize)]
pub struct NprPlaygroundSnapshot {
    pub revision: u64,
    pub settings: Settings,
    pub dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub scene: Option<String>,
    pub active_look: Option<String>,
    pub available_looks: Vec<String>,
    pub included_looks: Vec<String>,
    pub locked_layers: BTreeSet<String>,
    pub metadata: Value,
    pub brushes: amigo_render_npr::BrushLibrary,
    pub variants: BTreeMap<String, Settings>,
    pub drafts: BTreeMap<String, crate::documents::NprDrawingDraft>,
    pub layer_diagnostics: BTreeMap<String, amigo_render_npr::NprLayerDiagnostics>,
}

struct Session {
    camera_gesture: Option<(u64, bool)>,
    revision: u64,
    authored: Settings,
    baseline: Settings,
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    root: Option<PathBuf>,
    profile: Option<NprTrackedDocument>,
    active_look: Option<String>,
    baseline_look: Option<String>,
    look: Option<NprTrackedDocument>,
    events: Vec<PlaygroundEvent>,
    available_looks: Vec<String>,
    locked_layers: BTreeSet<String>,
    model_fingerprint: Option<Value>,
    variants: BTreeMap<String, Settings>,
    drafts: BTreeMap<String, crate::documents::NprDrawingDraft>,
}

#[derive(Clone)]
struct HistoryEntry {
    settings: Settings,
    active_look: Option<String>,
}

pub struct NprPlaygroundService {
    state: Arc<NprPlaygroundState>,
    session: Mutex<Session>,
    assets: Mutex<Option<Arc<amigo_assets::AssetCatalog>>>,
    render: Mutex<Option<Arc<crate::NprPlaygroundRenderService>>>,
    zoom: Mutex<crate::zoom::SmoothZoom>,
    imports:
        Mutex<Vec<std::sync::mpsc::Receiver<Result<crate::sources::NprImportedModel, String>>>>,
    catalogs: Mutex<Vec<NprCatalogSource>>,
    catalog_updates: Mutex<
        Vec<std::sync::mpsc::Receiver<(String, Result<crate::sources::NprRemoteCatalog, String>)>>,
    >,
    thumbnails: Mutex<BTreeMap<String, Option<String>>>,
    thumbnail_updates: Mutex<Vec<std::sync::mpsc::Receiver<(String, Option<String>)>>>,
    look_thumbnails: Mutex<BTreeMap<String, Option<String>>>,
    look_thumbnail_errors: Mutex<BTreeMap<String, String>>,
    look_thumbnail_retries: Mutex<BTreeMap<String, Instant>>,
    look_thumbnail_updates: Mutex<Vec<std::sync::mpsc::Receiver<(String, Result<String, String>)>>>,
    brush_thumbnails: Mutex<BTreeMap<String, Option<String>>>,
    brush_thumbnail_errors: Mutex<BTreeMap<String, String>>,
    brush_thumbnail_retries: Mutex<BTreeMap<String, Instant>>,
    brush_thumbnail_updates:
        Mutex<Vec<std::sync::mpsc::Receiver<(String, Result<String, String>)>>>,
}

#[derive(Clone, Serialize)]
struct NprCatalogSource {
    endpoint: crate::sources::NprCatalogEndpoint,
    state: String,
    models: Vec<crate::sources::NprRemoteModel>,
    error: Option<String>,
}

impl NprPlaygroundService {
    pub fn new(state: Arc<NprPlaygroundState>) -> Self {
        let settings = state.snapshot();
        Self {
            state,
            assets: Mutex::new(None),
            render: Mutex::new(None),
            zoom: Mutex::default(),
            imports: Mutex::default(),
            catalogs: Mutex::default(),
            catalog_updates: Mutex::default(),
            thumbnails: Mutex::default(),
            thumbnail_updates: Mutex::default(),
            look_thumbnails: Mutex::default(),
            look_thumbnail_errors: Mutex::default(),
            look_thumbnail_retries: Mutex::default(),
            look_thumbnail_updates: Mutex::default(),
            brush_thumbnails: Mutex::default(),
            brush_thumbnail_errors: Mutex::default(),
            brush_thumbnail_retries: Mutex::default(),
            brush_thumbnail_updates: Mutex::default(),
            session: Mutex::new(Session {
                camera_gesture: None,
                revision: 0,
                authored: settings.clone(),
                baseline: settings,
                undo: vec![],
                redo: vec![],
                root: None,
                profile: None,
                active_look: None,
                baseline_look: None,
                look: None,
                events: vec![],
                available_looks: vec![],
                locked_layers: BTreeSet::new(),
                model_fingerprint: None,
                variants: BTreeMap::new(),
                drafts: BTreeMap::new(),
            }),
        }
    }

    pub fn attach_runtime(
        &self,
        assets: Arc<amigo_assets::AssetCatalog>,
        render: Arc<crate::NprPlaygroundRenderService>,
    ) {
        *self.assets.lock().unwrap() = Some(assets);
        *self.render.lock().unwrap() = Some(render);
    }

    pub fn open_scene(&self, root: &Path, relative: &Path) -> Result<(), String> {
        let path = authored_path(root, relative)?;
        let tracked = NprTrackedDocument::open(path, true)?;
        let profile: NprSceneProfileDocument =
            serde_yaml::from_slice(&tracked.pending).map_err(|e| e.to_string())?;
        let looks = load_looks(root, profile.active_look.as_deref())?;
        let settings = profile.resolve(
            &looks,
            &load_model_defaults(root, &profile)?,
            &NprLookPatch::default(),
        )?;
        let mut restored_variants = BTreeMap::new();
        for (id, patch) in &profile.variants {
            let resolved = resolve_look(
                &BTreeMap::new(),
                &NprLookPatch::default(),
                None,
                patch,
                &NprLookPatch::default(),
                &NprLookPatch::default(),
            )?;
            let mut variant = settings.clone();
            variant.global = resolved.style;
            variant.style_layers = resolved.layers;
            restored_variants.insert(id.clone(), variant);
        }
        let look = profile
            .active_look
            .as_deref()
            .map(|id| {
                NprTrackedDocument::open(
                    authored_path(root, Path::new(&format!("npr/looks/{id}.npr-look.yml")))?,
                    true,
                )
            })
            .transpose()?;
        let catalog_path = authored_path(root, Path::new("npr/catalogs.yml"))?;
        let catalogs: crate::sources::NprCatalogsDocument = if catalog_path.exists() {
            serde_yaml::from_slice(&std::fs::read(catalog_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
        } else {
            Default::default()
        };
        *self.catalogs.lock().unwrap() = catalogs
            .catalogs
            .into_iter()
            .map(|endpoint| NprCatalogSource {
                endpoint,
                state: "unloaded".into(),
                models: vec![],
                error: None,
            })
            .collect();
        self.catalog_updates.lock().unwrap().clear();
        self.imports.lock().unwrap().clear();
        self.thumbnails.lock().unwrap().clear();
        self.thumbnail_updates.lock().unwrap().clear();
        self.look_thumbnails.lock().unwrap().clear();
        self.look_thumbnail_errors.lock().unwrap().clear();
        self.look_thumbnail_retries.lock().unwrap().clear();
        self.look_thumbnail_updates.lock().unwrap().clear();
        self.brush_thumbnails.lock().unwrap().clear();
        self.brush_thumbnail_errors.lock().unwrap().clear();
        self.brush_thumbnail_retries.lock().unwrap().clear();
        self.brush_thumbnail_updates.lock().unwrap().clear();
        if let (Some(assets), Some(render)) = (
            self.assets.lock().unwrap().clone(),
            self.render.lock().unwrap().clone(),
        ) {
            // Register the built-in catalog before the companion handshake.
            // Scene activation and the transport can complete in the same host
            // frame; waiting for the next Update would otherwise expose only
            // asynchronously rediscovered mod assets in the initial snapshot.
            crate::asset_browser::register_models(&assets, root);
            for model in crate::sources::imported_models(root)? {
                let key =
                    amigo_assets::AssetKey::new(format!("npr-playground/models/{}", model.id));
                assets.register_manifest(amigo_assets::AssetManifest {
                    key: key.clone(),
                    source: amigo_assets::AssetSourceKind::Mod(
                        root.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                    ),
                    tags: vec!["mesh-3d".into(), "npr-model".into()],
                });
                assets.begin_external_load(key.clone());
                let (tx, rx) = std::sync::mpsc::channel();
                self.imports.lock().unwrap().push(rx);
                let root = root.to_owned();
                let assets = assets.clone();
                let render = render.clone();
                std::thread::spawn(move || {
                    let result = render.load_model(&model.id, &model.path).map(|_| {
                        crate::asset_browser::register_imported(&assets, &root, &model);
                        model
                    });
                    if let Err(error) = &result {
                        assets.mark_failed(key, error);
                    }
                    let _ = tx.send(result);
                });
            }
        }
        let mut session = self.session.lock().unwrap();
        session.revision += 1;
        session.authored = settings.clone();
        session.baseline = settings.clone();
        session.undo.clear();
        session.redo.clear();
        session.root = Some(root.to_owned());
        session.profile = Some(tracked);
        session.look = look;
        session.baseline_look = profile.active_look.clone();
        session.active_look = profile.active_look;
        session.events.clear();
        session.locked_layers.clear();
        session.available_looks = available_looks(root)?;
        session.model_fingerprint = None;
        session.variants = restored_variants;
        session.drafts = load_drafts(root)?;
        session.camera_gesture = None;
        *self.zoom.lock().unwrap() = Default::default();
        *self.state.settings.lock().unwrap() = settings;
        self.state.set_build_up(1.0);
        self.state.set_solo_layer(None);
        Ok(())
    }

    pub fn metadata() -> Value {
        static METADATA: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        METADATA.get_or_init(Self::build_metadata).clone()
    }
    fn build_metadata() -> Value {
        // This describes the shipped Basic Mode, not the retired generic panel
        // editor. Layer and brush parameters stay typed document data until a
        // dedicated authoring surface ships; advertising them here recreated a
        // second, conflicting Look editor in generic clients.
        json!({
            "views": ["Sources", "Drawing", "Looks"],
            "controls": [
                {"id":"source.select","label":"Select source model","readonly":false,"disabled":false},
                {"id":"look.apply","label":"Apply saved look","readonly":false,"disabled":false},
                {"id":"drawing.reset-view","label":"Reset view","readonly":false,"disabled":false}
            ]
        })
    }

    /// Keep every source-changing intent on the same validation path.  The
    /// companion has both the Basic-mode source picker and typed object updates;
    /// accepting an arbitrary model through the latter would bypass the former.
    fn validate_source_model(&self, model: &str) -> Result<(), String> {
        if let Some(assets) = self.assets.lock().unwrap().as_ref() {
            if !crate::asset_browser::model_descriptors(assets)
                .iter()
                .any(|candidate| candidate.id == model && candidate.state == "ready")
            {
                return Err("model is not ready".into());
            }
        } else if !MODELS.contains(&model) {
            return Err(format!("unknown built-in source model `{model}`"));
        }
        Ok(())
    }

    pub fn domain_snapshot(&self) -> NprPlaygroundSnapshot {
        let s = self.session.lock().unwrap();
        let metadata = Self::metadata();
        NprPlaygroundSnapshot {
            revision: s.revision,
            settings: s.authored.clone(),
            dirty: s.authored != s.baseline || s.active_look != s.baseline_look,
            can_undo: !s.undo.is_empty(),
            can_redo: !s.redo.is_empty(),
            scene: s
                .profile
                .as_ref()
                .and_then(|p| p.path.parent())
                .and_then(|p| p.file_name())
                .map(|p| p.to_string_lossy().into_owned()),
            active_look: s.active_look.clone(),
            available_looks: s.available_looks.clone(),
            included_looks: s
                .look
                .as_ref()
                .and_then(|doc| serde_yaml::from_slice::<NprLookDocument>(&doc.pending).ok())
                .map(|look| look.includes)
                .unwrap_or_default(),
            locked_layers: s.locked_layers.clone(),
            metadata,
            brushes: s.authored.brushes.clone(),
            variants: s.variants.clone(),
            drafts: s.drafts.clone(),
            layer_diagnostics: self
                .render
                .lock()
                .unwrap()
                .as_ref()
                .map(|render| render.layer_diagnostics())
                .unwrap_or_default(),
        }
    }

    pub fn advance_camera(&self, seconds: f32) {
        let session = self.session.lock().unwrap();
        if session.profile.is_none() {
            return;
        }
        let mut runtime = self.state.settings.lock().unwrap();
        let distance = runtime.camera_distance;
        let next = self.zoom.lock().unwrap().advance(distance, 0.0, seconds);
        // Interpolation is presentation state. The authored target/revision
        // changes only for a user action, so easing cannot reject queued input.
        runtime.camera_distance = next;
    }

    pub fn dispatch_intent(
        &self,
        request_id: u64,
        base_revision: u64,
        control: String,
        intent: NprPlaygroundIntent,
    ) -> Result<u64, NprPlaygroundActionError> {
        let mut s = self.session.lock().unwrap();
        let error = |code: &str, message: String| NprPlaygroundActionError {
            control: control.clone(),
            code: code.into(),
            message,
        };
        if s.revision != base_revision {
            s.camera_gesture = None;
            return Err(error(
                "revision_conflict",
                "State changed; refresh before retrying".into(),
            ));
        }
        let before = s.authored.clone();
        let history_before = HistoryEntry {
            settings: before.clone(),
            active_look: s.active_look.clone(),
        };
        let save_operation = matches!(
            &intent,
            NprPlaygroundIntent::SaveAll
                | NprPlaygroundIntent::SaveLook
                | NprPlaygroundIntent::SaveAsLook { .. }
        );
        let mut next = before.clone();
        let mut record = true;
        let mut explicit_pose = None;
        let mut smoothed_distance = None;
        let mut event = None;
        let navigation = matches!(&intent, NprPlaygroundIntent::Navigate { .. });
        if !navigation
            && !matches!(
                &intent,
                NprPlaygroundIntent::BeginCameraGesture { .. }
                    | NprPlaygroundIntent::EndCameraGesture { .. }
            )
        {
            s.camera_gesture = None;
        }
        let result: Result<(), String> = (|| {
            match intent {
                NprPlaygroundIntent::BeginCameraGesture { gesture_id } => {
                    s.camera_gesture = Some((gesture_id, false));
                    record = false;
                }
                NprPlaygroundIntent::EndCameraGesture { gesture_id } => {
                    if s.camera_gesture.is_some_and(|(id, _)| id == gesture_id) {
                        s.camera_gesture = None;
                    }
                    record = false;
                }
                NprPlaygroundIntent::SetSeed { seed } => next.seed = seed,
                NprPlaygroundIntent::SetBuildUp { value } => {
                    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                        return Err("build-up must be between 0 and 1".into());
                    }
                    self.state.set_build_up(value);
                    record = false;
                }
                NprPlaygroundIntent::SetSoloLayer { layer } => {
                    if let Some(layer_id) = &layer {
                        if s.authored.style_layers.layer(layer_id).is_none() {
                            return Err(format!("unknown layer `{layer_id}`"));
                        }
                    }
                    self.state.set_solo_layer(layer);
                    record = false;
                }
                NprPlaygroundIntent::SaveBrushVersion { brush } => {
                    brush.validate()?;
                    let next_version = s
                        .authored
                        .brushes
                        .brushes
                        .get(&brush.id)
                        .and_then(|items| items.iter().map(|item| item.version).max())
                        .unwrap_or(0)
                        + 1;
                    if brush.version != next_version {
                        return Err(format!("new brush version must be {}", next_version));
                    }
                    next.brushes.add_version(brush)?;
                    event = Some("brush_version_saved");
                }
                NprPlaygroundIntent::SaveVariant { id } => {
                    validate_document_id(&id)?;
                    if s.variants.len() >= 32 && !s.variants.contains_key(&id) {
                        return Err("variant limit reached".into());
                    }
                    s.variants.insert(id, next.clone());
                    record = false;
                }
                NprPlaygroundIntent::ApplyVariant { id } => {
                    next = s
                        .variants
                        .get(&id)
                        .cloned()
                        .ok_or_else(|| format!("unknown variant `{id}`"))?;
                    event = Some("variant_applied");
                }
                NprPlaygroundIntent::UpdateBrushVersion { id, from, to } => {
                    // Resolve against the document's active library so local
                    // brush versions can be updated exactly like built-ins.
                    // The library itself is immutable here; only matching
                    // layer references are changed as one history operation.
                    let reference = amigo_render_npr::BrushReference {
                        id: id.clone(),
                        version: to,
                    };
                    if next.brushes.resolve(&reference).is_err() {
                        // A fresh in-memory service has no persisted library
                        // yet; built-ins remain available as read-only
                        // resources in that state.
                        builtin_brush_library().resolve(&reference)?;
                    }
                    let mut updated = 0usize;
                    for layer in &mut next.style_layers.layers {
                        if layer.brush.as_ref().is_some_and(|instance| {
                            instance.brush.id == id && instance.brush.version == from
                        }) {
                            layer.brush.as_mut().expect("checked above").brush.version = to;
                            updated += 1;
                        }
                    }
                    if updated == 0 {
                        return Err(format!("no layers use brush {id}@{from}"));
                    }
                    event = Some("brush_version_updated");
                }
                NprPlaygroundIntent::SetTemporalPolicy { policy } => next.motion = policy,
                NprPlaygroundIntent::SetDebugView { view } => next.debug = view,
                NprPlaygroundIntent::SetLayerLock { layer, locked } => {
                    if next.style_layers.layer(&layer).is_none() {
                        return Err("unknown layer".into());
                    }
                    if locked {
                        s.locked_layers.insert(layer);
                    } else {
                        s.locked_layers.remove(&layer);
                    }
                    record = false;
                }
                NprPlaygroundIntent::UseLook { id } => {
                    let root = s.root.as_ref().ok_or("no active mod")?;
                    let looks = load_looks(root, Some(&id))?;
                    let empty = NprLookPatch::default();
                    let resolved = resolve_look(&looks, &empty, Some(&id), &empty, &empty, &empty)?;
                    next.global = resolved.style;
                    next.style_layers = resolved.layers;
                    next.validate()?;
                    let doc = NprTrackedDocument::open(
                        authored_path(root, Path::new(&format!("npr/looks/{id}.npr-look.yml")))?,
                        true,
                    )?;
                    s.active_look = Some(id);
                    s.look = Some(doc);
                    event = Some("look_changed");
                }
                NprPlaygroundIntent::RefreshCatalog { catalog } => {
                    let mut catalogs = self.catalogs.lock().unwrap();
                    let source = catalogs
                        .iter_mut()
                        .find(|c| c.endpoint.id == catalog)
                        .ok_or("untrusted catalog")?;
                    if source.state == "loading" {
                        return Err("catalog is already loading".into());
                    }
                    source.state = "loading".into();
                    source.error = None;
                    let endpoint = source.endpoint.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.catalog_updates.lock().unwrap().push(rx);
                    std::thread::spawn(move || {
                        let result = crate::sources::fetch_catalog(&endpoint);
                        let _ = tx.send((endpoint.id, result));
                    });
                    record = false;
                }
                NprPlaygroundIntent::ImportRemote { catalog, model } => {
                    if !self.imports.lock().unwrap().is_empty() {
                        return Err("an import is already running".into());
                    }
                    let root = s.root.clone().ok_or("no active mod")?;
                    let catalogs = self.catalogs.lock().unwrap();
                    let source = catalogs
                        .iter()
                        .find(|c| c.endpoint.id == catalog)
                        .ok_or("untrusted catalog")?;
                    let model = source
                        .models
                        .iter()
                        .find(|m| m.id == model)
                        .cloned()
                        .ok_or("model not present in verified catalog")?;
                    let endpoint = source.endpoint.clone();
                    let assets = self
                        .assets
                        .lock()
                        .unwrap()
                        .clone()
                        .ok_or("asset catalog unavailable")?;
                    let render = self
                        .render
                        .lock()
                        .unwrap()
                        .clone()
                        .ok_or("render service unavailable")?;
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.imports.lock().unwrap().push(rx);
                    std::thread::spawn(move || {
                        let result = crate::sources::import_remote(
                            &root,
                            &root.join(".cache/npr"),
                            &endpoint,
                            &model,
                        )
                        .and_then(|model| {
                            render.load_model(&model.id, &model.path)?;
                            crate::asset_browser::register_imported(&assets, &root, &model);
                            Ok(model)
                        });
                        let _ = tx.send(result);
                    });
                    record = false;
                    event = Some("model_loading");
                }
                NprPlaygroundIntent::ImportModel { path } => {
                    if !self.imports.lock().unwrap().is_empty() {
                        return Err("an import is already running".into());
                    }
                    let root = s.root.clone().ok_or("no active mod")?;
                    let assets = self
                        .assets
                        .lock()
                        .unwrap()
                        .clone()
                        .ok_or("asset catalog unavailable")?;
                    let render = self
                        .render
                        .lock()
                        .unwrap()
                        .clone()
                        .ok_or("render service unavailable")?;
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.imports.lock().unwrap().push(rx);
                    std::thread::spawn(move || {
                        let result = crate::sources::import_local(&root, &path).and_then(|model| {
                            render.load_model(&model.id, &model.path)?;
                            crate::asset_browser::register_imported(&assets, &root, &model);
                            Ok(model)
                        });
                        let _ = tx.send(result);
                    });
                    record = false;
                    event = Some("model_loading");
                }
                NprPlaygroundIntent::Navigate {
                    mode,
                    dx,
                    dy,
                    wheel,
                    x,
                    y,
                    width,
                    height,
                    focused,
                } => {
                    if !focused {
                        return Err("viewport does not have focus".into());
                    }
                    if [dx, dy, wheel, x, y].iter().any(|v| !v.is_finite())
                        || width == 0
                        || height == 0
                    {
                        return Err("invalid viewport input".into());
                    }
                    match mode.as_str() {
                        "orbit" => {
                            next.camera_yaw += dx * 0.3;
                            next.camera_pitch = (next.camera_pitch + dy * 0.3).clamp(-89.0, 89.0);
                        }
                        "pan" => {
                            let yaw = next.camera_yaw.to_radians();
                            let right = glam::Vec3::new(yaw.cos(), 0.0, -yaw.sin());
                            let scale = next.camera_distance
                                * 2.0
                                * (next.camera_fov.to_radians() * 0.5).tan()
                                / height as f32;
                            next.camera_target += (-right * dx + glam::Vec3::Y * dy) * scale;
                        }
                        "zoom" => {
                            let current = self.state.settings.lock().unwrap().camera_distance;
                            let mut zoom = self.zoom.lock().unwrap();
                            smoothed_distance = Some(zoom.advance(current, wheel, 1.0 / 60.0));
                            next.camera_distance = zoom.target_distance();
                        }
                        "fit" | "focus" => {
                            NprPlaygroundState::fit_candidate(&mut next, [width, height])?;
                        }
                        "reset" => {
                            next.camera_target = glam::Vec3::ZERO;
                            next.camera_yaw = 0.0;
                            next.camera_pitch = 0.0;
                            next.camera_distance = 5.0;
                        }
                        "select" => {
                            let render = self.render.lock().unwrap();
                            let render = render.as_ref().ok_or("render service unavailable")?;
                            if let Some(pick) = render.pick_surface(
                                &self.state.snapshot(),
                                [width, height],
                                glam::Vec2::new(x, y),
                            ) {
                                next.selected = pick.object_id;
                                event = Some("selection_changed");
                            }
                        }
                        _ => return Err("unknown camera navigation mode".into()),
                    }
                }
                NprPlaygroundIntent::Select { object } => {
                    next.selected = object;
                    event = Some("selection_changed");
                }
                NprPlaygroundIntent::SetCamera {
                    target,
                    yaw,
                    pitch,
                    distance,
                    fov,
                } => {
                    next.camera_target = target;
                    next.camera_yaw = yaw;
                    next.camera_pitch = pitch;
                    next.camera_distance = distance;
                    next.camera_fov = fov;
                }
                NprPlaygroundIntent::SetMotion {
                    paused,
                    speed,
                    sketch_paused,
                } => {
                    next.paused = paused;
                    next.speed = speed;
                    next.sketch_paused = sketch_paused;
                }
                NprPlaygroundIntent::SetObjectPose {
                    object,
                    position,
                    rotation,
                    scale,
                } => {
                    let target = next.objects.get_mut(&object).ok_or("unknown object")?;
                    target.position = position;
                    target.rotation = rotation;
                    target.scale = scale;
                    explicit_pose = Some(object);
                }
                NprPlaygroundIntent::SetObject { object, settings } => {
                    let source_changed = next
                        .objects
                        .get(&object)
                        .ok_or("unknown object")?
                        .model
                        != settings.model;
                    if source_changed {
                        return Err("change the Drawing Studio source with SelectModel".into());
                    }
                    let existing = next.objects.get_mut(&object).ok_or("unknown object")?;
                    *existing = settings;
                }
                NprPlaygroundIntent::SetLook { style, layers } => {
                    for id in &s.locked_layers {
                        let old = next
                            .style_layers
                            .layers
                            .iter()
                            .position(|layer| &layer.id == id);
                        let new = layers.layers.iter().position(|layer| &layer.id == id);
                        if old != new || next.style_layers.layer(id) != layers.layer(id) {
                            return Err(format!("layer is locked: {id}"));
                        }
                    }
                    next.global = style;
                    next.style_layers = layers;
                    event = Some("look_changed");
                }
                NprPlaygroundIntent::SelectModel { model } => {
                    self.validate_source_model(&model)?;
                    if model != next.selected && next != s.baseline {
                        let draft = crate::documents::NprDrawingDraft {
                            version: 1,
                            source_model: next.selected.clone(),
                            settings: next.clone(),
                        };
                        if let Some(root) = s.root.as_deref() {
                            validate_document_id(&draft.source_model)?;
                            let path = authored_path(
                                root,
                                Path::new(&format!(
                                    "npr/drafts/{}.npr-draft.yml",
                                    draft.source_model
                                )),
                            )?;
                            std::fs::create_dir_all(
                                path.parent().ok_or("draft path has no parent")?,
                            )
                            .map_err(|error| format!("create local draft directory: {error}"))?;
                            let mut document = NprTrackedDocument::new(path, Vec::new());
                            document.stage(&draft)?;
                            document.save()?;
                        }
                        s.drafts.insert(draft.source_model.clone(), draft);
                    }
                    let defaults = Settings::for_scene();
                    let template = if let Some(template) = defaults.objects.get(&model) {
                        template.clone()
                    } else {
                        let mut template = defaults
                            .objects
                            .get(&defaults.selected)
                            .ok_or("Drawing Studio default source model is missing")?
                            .clone();
                        template.model = model.clone();
                        template.material_base_color = glam::Vec4::ONE;
                        template
                    };
                    // Sources is a model explorer. Selecting a different
                    // subject starts a clean, model-bound Drawing Studio
                    // document; the previous document was checkpointed above.
                    next = defaults;
                    next.objects.clear();
                    let id = model.clone();
                    next.objects.insert(id.clone(), template);
                    next.selected = id;
                    s.active_look = None;
                    s.baseline_look = None;
                    s.variants.clear();
                    s.locked_layers.clear();
                    s.baseline = next.clone();
                    s.undo.clear();
                    s.redo.clear();
                    record = false;
                    event = Some("selection_changed");
                }
                NprPlaygroundIntent::OpenDraft { model } => {
                    let draft = s
                        .drafts
                        .get(&model)
                        .cloned()
                        .ok_or_else(|| format!("no local draft for model `{model}`"))?;
                    if draft.source_model != model {
                        return Err("draft source model does not match its identifier".into());
                    }
                    draft.settings.validate()?;
                    next = draft.settings;
                    s.baseline = next.clone();
                    s.active_look = None;
                    s.baseline_look = None;
                    s.undo.clear();
                    s.redo.clear();
                    s.locked_layers.clear();
                    record = false;
                    event = Some("draft_opened");
                }
                NprPlaygroundIntent::RemoveObject { object } => {
                    next.objects.remove(&object).ok_or("unknown object")?;
                    if next.selected == object {
                        next.selected = next.objects.keys().next().cloned().unwrap_or_default();
                    }
                    event = Some("selection_changed");
                }
                NprPlaygroundIntent::Undo => {
                    let entry = s.undo.last().cloned().ok_or("nothing to undo")?;
                    next = entry.settings;
                    next.validate()?;
                    let look = tracked_look(s.root.as_deref(), entry.active_look.as_deref())?;
                    s.active_look = entry.active_look;
                    s.look = look;
                    s.undo.pop();
                    s.redo.push(history_before.clone());
                    record = false;
                }
                NprPlaygroundIntent::Redo => {
                    let entry = s.redo.last().cloned().ok_or("nothing to redo")?;
                    next = entry.settings;
                    next.validate()?;
                    let look = tracked_look(s.root.as_deref(), entry.active_look.as_deref())?;
                    s.active_look = entry.active_look;
                    s.look = look;
                    s.redo.pop();
                    s.undo.push(history_before.clone());
                    record = false;
                }
                NprPlaygroundIntent::SaveAll => {
                    let mut profile =
                        NprSceneProfileDocument::from_settings(&next, s.active_look.clone())?;
                    let old: NprSceneProfileDocument = serde_yaml::from_slice(
                        &s.profile.as_ref().ok_or("no scene profile")?.pending,
                    )
                    .map_err(|e| e.to_string())?;
                    profile.look = old.look;
                    profile.look.merge(&NprLookPatch::changes(
                        &NprResolvedLook {
                            style: s.baseline.global,
                            layers: s.baseline.style_layers.clone(),
                        },
                        &NprResolvedLook {
                            style: next.global,
                            layers: next.style_layers.clone(),
                        },
                    )?)?;
                    profile.variants = s
                        .variants
                        .iter()
                        .map(|(id, settings)| {
                            Ok((
                                id.clone(),
                                NprLookPatch::from_resolved(&NprResolvedLook {
                                    style: settings.global,
                                    layers: settings.style_layers.clone(),
                                })?,
                            ))
                        })
                        .collect::<Result<_, String>>()?;
                    profile.brushes = next.brushes.clone();
                    let mut profile_document = s
                        .profile
                        .as_ref()
                        .ok_or("no writable scene profile")?
                        .clone();
                    profile_document.stage(&profile)?;
                    let save_look = s.look.as_ref().filter(|look| look.dirty).cloned();
                    let save_look_pending = save_look.is_some();
                    let mut documents = vec![profile_document];
                    if let Some(look) = save_look {
                        documents.push(look);
                    }
                    save_all(&mut documents)?;
                    s.profile = Some(documents.remove(0));
                    if save_look_pending {
                        s.look = Some(documents.remove(0));
                    }
                    s.baseline_look = s.active_look.clone();
                    s.baseline = next.clone();
                    s.undo.clear();
                    s.redo.clear();
                    record = false;
                    event = Some("save_completed");
                }
                NprPlaygroundIntent::SaveLook => {
                    let changes = NprLookPatch::changes(
                        &NprResolvedLook {
                            style: s.baseline.global,
                            layers: s.baseline.style_layers.clone(),
                        },
                        &NprResolvedLook {
                            style: next.global,
                            layers: next.style_layers.clone(),
                        },
                    )?;
                    let doc = s
                        .look
                        .as_mut()
                        .ok_or("no active writable look; use Save As Look")?;
                    let mut look: NprLookDocument =
                        serde_yaml::from_slice(&doc.pending).map_err(|e| e.to_string())?;
                    look.look.merge(&changes)?;
                    doc.stage(&look)?;
                    doc.save()?;
                    s.baseline.global = next.global;
                    s.baseline.style_layers = next.style_layers.clone();
                    s.undo.clear();
                    s.redo.clear();
                    record = false;
                    event = Some("save_completed");
                }
                NprPlaygroundIntent::SaveAsLook { id } => {
                    validate_document_id(&id)?;
                    let root = s.root.as_ref().ok_or("no active mod")?;
                    let path =
                        authored_path(root, Path::new(&format!("npr/looks/{id}.npr-look.yml")))?;
                    let look = NprLookDocument {
                        id: id.clone(),
                        includes: vec![],
                        look: NprLookPatch::from_resolved(&NprResolvedLook {
                            style: next.global,
                            layers: next.style_layers.clone(),
                        })?,
                    };
                    let mut doc = NprTrackedDocument::new(path, vec![]);
                    doc.stage(&look)?;
                    doc.save()?;
                    s.available_looks.push(id.clone());
                    s.available_looks.sort();
                    s.active_look = Some(id);
                    s.look = Some(doc);
                    s.undo.clear();
                    s.redo.clear();
                    record = false;
                    event = Some("save_completed");
                }
                NprPlaygroundIntent::Reload => {
                    let root = s.root.clone().ok_or("no active mod")?;
                    let doc = s.profile.as_ref().ok_or("no scene profile")?;
                    let fresh = NprTrackedDocument::open(doc.path.clone(), doc.writable)?;
                    let profile: NprSceneProfileDocument =
                        serde_yaml::from_slice(&fresh.pending).map_err(|e| e.to_string())?;
                    next = profile.resolve(
                        &load_looks(&root, profile.active_look.as_deref())?,
                        &load_model_defaults(&root, &profile)?,
                        &NprLookPatch::default(),
                    )?;
                    s.look = profile
                        .active_look
                        .as_deref()
                        .map(|id| {
                            NprTrackedDocument::open(
                                authored_path(
                                    &root,
                                    Path::new(&format!("npr/looks/{id}.npr-look.yml")),
                                )?,
                                true,
                            )
                        })
                        .transpose()?;
                    s.profile = Some(fresh);
                    s.baseline = next.clone();
                    s.baseline_look = profile.active_look.clone();
                    s.active_look = profile.active_look;
                    s.undo.clear();
                    s.redo.clear();
                    record = false;
                }
            }
            next.validate()
        })();
        if let Err(message) = result {
            s.events.push(PlaygroundEvent::Domain {
                name: if save_operation {
                    "save_failed"
                } else {
                    "action_failed"
                }
                .into(),
                payload: json!({"request_id":request_id,"control":control,"message":message}),
            });
            return Err(error("invalid_action", message));
        }
        if record && (next != before || s.active_look != history_before.active_look) {
            let recorded = navigation && s.camera_gesture.is_some_and(|(_, recorded)| recorded);
            if !recorded {
                s.undo.push(history_before);
            }
            if navigation {
                if let Some((_, recorded)) = &mut s.camera_gesture {
                    *recorded = true;
                }
            }
            if s.undo.len() > 128 {
                s.undo.remove(0);
            }
            s.redo.clear();
        }
        s.authored = next.clone();
        s.revision += 1;
        let mut runtime_settings = self.state.settings.lock().unwrap();
        if let Some(distance) = smoothed_distance {
            next.camera_distance = distance;
        } else if next.camera_distance == before.camera_distance {
            next.camera_distance = runtime_settings.camera_distance;
        }
        for (id, object) in &mut next.objects {
            if explicit_pose.as_ref() == Some(id) {
                continue;
            }
            if before
                .objects
                .get(id)
                .is_some_and(|old| old.rotation == object.rotation)
            {
                if let Some(live) = runtime_settings.objects.get(id) {
                    object.rotation = live.rotation;
                }
            }
        }
        *runtime_settings = next;
        if let Some(name) = event {
            s.events.push(PlaygroundEvent::Domain {
                name: name.into(),
                payload: json!({"request_id":request_id}),
            });
        }
        Ok(s.revision)
    }
}

fn tracked_look(
    root: Option<&Path>,
    id: Option<&str>,
) -> Result<Option<NprTrackedDocument>, String> {
    id.map(|id| {
        NprTrackedDocument::open(
            authored_path(
                root.ok_or("no active mod")?,
                Path::new(&format!("npr/looks/{id}.npr-look.yml")),
            )?,
            true,
        )
    })
    .transpose()
}

fn load_model_defaults(
    root: &Path,
    profile: &NprSceneProfileDocument,
) -> Result<BTreeMap<String, NprLookPatch>, String> {
    let mut defaults = BTreeMap::new();
    for object in profile.objects.values() {
        validate_document_id(&object.model)?;
        if defaults.contains_key(&object.model) {
            continue;
        }
        let path = authored_path(
            root,
            Path::new(&format!("assets/models/{}/npr.model.yml", object.model)),
        )?;
        if path.exists() {
            let document: NprModelOverrideDocument =
                serde_yaml::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            defaults.insert(object.model.clone(), document.look);
        }
    }
    Ok(defaults)
}

fn load_drafts(root: &Path) -> Result<BTreeMap<String, crate::documents::NprDrawingDraft>, String> {
    let folder = authored_path(root, Path::new("npr/drafts"))?;
    if !folder.exists() {
        return Ok(BTreeMap::new());
    }
    let mut drafts = BTreeMap::new();
    for entry in std::fs::read_dir(&folder).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("yml") {
            continue;
        }
        let draft: crate::documents::NprDrawingDraft =
            serde_yaml::from_slice(&std::fs::read(&path).map_err(|error| error.to_string())?)
                .map_err(|error| format!("{}: {error}", path.display()))?;
        validate_document_id(&draft.source_model)?;
        draft.settings.validate()?;
        if drafts.insert(draft.source_model.clone(), draft).is_some() {
            return Err(format!(
                "duplicate NPR drawing draft in {}",
                folder.display()
            ));
        }
    }
    Ok(drafts)
}

fn available_looks(root: &Path) -> Result<Vec<String>, String> {
    let folder = authored_path(root, Path::new("npr/looks"))?;
    if !folder.exists() {
        return Ok(vec![]);
    }
    let mut ids = Vec::new();
    for file in std::fs::read_dir(folder).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        if let Some(id) = file
            .file_name()
            .to_str()
            .and_then(|name| name.strip_suffix(".npr-look.yml"))
        {
            validate_document_id(id)?;
            ids.push(id.to_owned());
        }
    }
    ids.sort();
    Ok(ids)
}

fn load_looks(
    root: &Path,
    active: Option<&str>,
) -> Result<BTreeMap<String, NprLookDocument>, String> {
    fn visit(
        root: &Path,
        id: &str,
        looks: &mut BTreeMap<String, NprLookDocument>,
    ) -> Result<(), String> {
        validate_document_id(id)?;
        if looks.contains_key(id) {
            return Ok(());
        }
        if looks.len() >= 256 {
            return Err("too many included looks".into());
        }
        let path = authored_path(root, Path::new(&format!("npr/looks/{id}.npr-look.yml")))?;
        let look: NprLookDocument =
            serde_yaml::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let includes = look.includes.clone();
        looks.insert(id.into(), look);
        for include in includes {
            visit(root, &include, looks)?;
        }
        Ok(())
    }
    let mut looks = BTreeMap::new();
    if let Some(id) = active {
        visit(root, id, &mut looks)?;
    }
    Ok(looks)
}

/// Resolves a reusable Look without a scene document.  Preview generation uses
/// this exact authoring path, so includes, pinned brushes and layer ordering
/// cannot diverge between a card and the drawing viewport.
pub(crate) fn resolve_preview_look(
    root: &Path,
    id: &str,
) -> Result<crate::documents::NprResolvedLook, String> {
    let looks = load_looks(root, Some(id))?;
    resolve_look(
        &looks,
        &NprLookPatch::default(),
        Some(id),
        &NprLookPatch::default(),
        &NprLookPatch::default(),
        &NprLookPatch::default(),
    )
}

impl PlaygroundProvider for NprPlaygroundService {
    fn revision(&self) -> u64 {
        self.session.lock().unwrap().revision
    }
    fn cancel_interaction(&self) {
        self.session.lock().unwrap().camera_gesture = None;
    }
    fn open_scene(&self, root: &Path, scene: &Path) -> Result<(), String> {
        let value: serde_yaml::Value =
            serde_yaml::from_slice(&std::fs::read(scene).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let profile = value
            .get("entities")
            .and_then(|v| v.as_sequence())
            .into_iter()
            .flatten()
            .flat_map(|entity| {
                entity
                    .get("components")
                    .and_then(|v| v.as_sequence())
                    .into_iter()
                    .flatten()
            })
            .find(|c| {
                c.get("type").and_then(|v| v.as_str())
                    == Some(crate::scene::NPR_SETTINGS_COMPONENT_TYPE)
            })
            .and_then(|c| c.get("profile"))
            .and_then(|v| v.as_str())
            .ok_or("NprSettings requires a profile reference")?;
        let scene_relative = scene.strip_prefix(root).map_err(|e| e.to_string())?;
        let relative = scene_relative
            .parent()
            .ok_or("scene has no directory")?
            .join(profile);
        let path = authored_path(root, &relative)?;
        if self
            .session
            .lock()
            .unwrap()
            .profile
            .as_ref()
            .is_some_and(|doc| doc.path == path)
        {
            return Ok(());
        }
        NprPlaygroundService::open_scene(self, root, &relative)
    }
    fn descriptor(&self) -> PlaygroundDescriptor {
        PlaygroundDescriptor {
            id: PlaygroundId("npr-playground".into()),
            label: "NprPlayground".into(),
            client: "npr-playground".into(),
        }
    }
    fn snapshot(&self) -> PlaygroundSnapshot {
        let mut thumbnails = Vec::new();
        self.thumbnail_updates
            .lock()
            .unwrap()
            .retain(|rx| match rx.try_recv() {
                Ok(value) => {
                    thumbnails.push(value);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(_) => false,
            });
        if !thumbnails.is_empty() {
            self.thumbnails.lock().unwrap().extend(thumbnails);
            self.session.lock().unwrap().revision += 1;
        }
        let mut look_thumbnails = Vec::new();
        self.look_thumbnail_updates
            .lock()
            .unwrap()
            .retain(|rx| match rx.try_recv() {
                Ok(value) => {
                    look_thumbnails.push(value);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(_) => false,
            });
        if !look_thumbnails.is_empty() {
            let mut previews = self.look_thumbnails.lock().unwrap();
            let mut errors = self.look_thumbnail_errors.lock().unwrap();
            let mut retries = self.look_thumbnail_retries.lock().unwrap();
            for (id, result) in look_thumbnails {
                match result {
                    Ok(preview) => {
                        previews.insert(id.clone(), Some(preview));
                        errors.remove(&id);
                        retries.remove(&id);
                    }
                    Err(error) => {
                        previews.insert(id.clone(), None);
                        retries.insert(id.clone(), Instant::now() + Duration::from_secs(2));
                        errors.insert(id, error);
                    }
                }
            }
            self.session.lock().unwrap().revision += 1;
        }
        let mut brush_thumbnails = Vec::new();
        self.brush_thumbnail_updates
            .lock()
            .unwrap()
            .retain(|rx| match rx.try_recv() {
                Ok(value) => {
                    brush_thumbnails.push(value);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(_) => false,
            });
        if !brush_thumbnails.is_empty() {
            let mut previews = self.brush_thumbnails.lock().unwrap();
            let mut errors = self.brush_thumbnail_errors.lock().unwrap();
            let mut retries = self.brush_thumbnail_retries.lock().unwrap();
            for (id, result) in brush_thumbnails {
                match result {
                    Ok(preview) => {
                        previews.insert(id.clone(), Some(preview));
                        errors.remove(&id);
                        retries.remove(&id);
                    }
                    Err(error) => {
                        previews.insert(id.clone(), None);
                        retries.insert(id.clone(), Instant::now() + Duration::from_secs(2));
                        errors.insert(id, error);
                    }
                }
            }
            self.session.lock().unwrap().revision += 1;
        }
        let mut updates = Vec::new();
        self.catalog_updates
            .lock()
            .unwrap()
            .retain(|rx| match rx.try_recv() {
                Ok(update) => {
                    updates.push(update);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(_) => false,
            });
        if !updates.is_empty() {
            let mut session = self.session.lock().unwrap();
            let mut catalogs = self.catalogs.lock().unwrap();
            for (id, result) in updates {
                if let Some(source) = catalogs.iter_mut().find(|c| c.endpoint.id == id) {
                    match result {
                        Ok(catalog) => {
                            source.state = "ready".into();
                            source.models = catalog.models;
                            source.error = None;
                        }
                        Err(error) => {
                            source.state = "failed".into();
                            source.error = Some(error);
                        }
                    }
                    session.revision += 1;
                }
            }
        }
        let mut finished = Vec::new();
        self.imports
            .lock()
            .unwrap()
            .retain(|rx| match rx.try_recv() {
                Ok(result) => {
                    finished.push(result);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(_) => {
                    finished.push(Err("import worker stopped".into()));
                    false
                }
            });
        if !finished.is_empty() {
            let mut session = self.session.lock().unwrap();
            for result in finished {
                let (name, payload) = match result {
                    Ok(model) => ("model_loaded", json!({"id":model.id})),
                    Err(error) => ("model_failed", json!({"message":error})),
                };
                session.events.push(PlaygroundEvent::Domain {
                    name: name.into(),
                    payload,
                });
                session.revision += 1;
            }
        }
        let snapshot = self.domain_snapshot();
        let assets = self.assets.lock().unwrap().clone();
        let mut models = assets
            .as_ref()
            .map(|assets| crate::asset_browser::model_descriptors(assets))
            .unwrap_or_default();
        let root = self.session.lock().unwrap().root.clone();
        let mut previews = self.look_thumbnails.lock().unwrap();
        let mut errors = self.look_thumbnail_errors.lock().unwrap();
        let mut retries = self.look_thumbnail_retries.lock().unwrap();
        for id in &snapshot.available_looks {
            let retry_ready = retries.get(id).is_some_and(|retry| *retry <= Instant::now());
            if (!previews.contains_key(id) || retry_ready)
                && self.look_thumbnail_updates.lock().unwrap().len() < 2
            {
                if let Some(root) = root.clone() {
                    previews.insert(id.clone(), None);
                    errors.remove(id);
                    retries.remove(id);
                    let id = id.clone();
                    let brushes = snapshot.brushes.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.look_thumbnail_updates.lock().unwrap().push(rx);
                    std::thread::spawn(move || {
                        let thumbnail = crate::asset_browser::look_thumbnail(&root, &id, &brushes);
                        let _ = tx.send((id, thumbnail));
                    });
                }
            }
        }
        let look_previews = previews.clone();
        drop(previews);
        drop(errors);
        drop(retries);
        let look_preview_errors = self.look_thumbnail_errors.lock().unwrap().clone();
        let mut brush_previews = self.brush_thumbnails.lock().unwrap();
        let mut brush_errors = self.brush_thumbnail_errors.lock().unwrap();
        let mut brush_retries = self.brush_thumbnail_retries.lock().unwrap();
        for brush in snapshot.brushes.brushes.values().flatten() {
            let key = format!("{}@{}", brush.id, brush.version);
            let retry_ready = brush_retries
                .get(&key)
                .is_some_and(|retry| *retry <= Instant::now());
            if (!brush_previews.contains_key(&key) || retry_ready)
                && self.brush_thumbnail_updates.lock().unwrap().len() < 2
            {
                if let Some(root) = root.clone() {
                    brush_previews.insert(key.clone(), None);
                    brush_errors.remove(&key);
                    brush_retries.remove(&key);
                    let brush = brush.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.brush_thumbnail_updates.lock().unwrap().push(rx);
                    std::thread::spawn(move || {
                        let thumbnail = crate::asset_browser::brush_thumbnail(&root, &brush);
                        let _ = tx.send((key, thumbnail));
                    });
                }
            }
        }
        let brush_preview_values = brush_previews.clone();
        drop(brush_previews);
        drop(brush_errors);
        drop(brush_retries);
        let brush_preview_errors = self.brush_thumbnail_errors.lock().unwrap().clone();
        let mut thumbnails = self.thumbnails.lock().unwrap();
        for model in &mut models {
            if model.state == "ready"
                && !thumbnails.contains_key(&model.id)
                && self.thumbnail_updates.lock().unwrap().len() < 2
            {
                if let Some(root) = root.clone() {
                    let id = model.id.clone();
                    thumbnails.insert(id.clone(), None);
                    let path = assets
                        .as_ref()
                        .and_then(|a| {
                            a.prepared_asset(&amigo_assets::AssetKey::new(format!(
                                "npr-playground/models/{id}"
                            )))
                        })
                        .filter(|a| a.format.as_deref() == Some("gltf"))
                        .map(|a| a.resolved_path);
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.thumbnail_updates.lock().unwrap().push(rx);
                    std::thread::spawn(move || {
                        let thumbnail =
                            crate::asset_browser::thumbnail(&root, &id, path.as_deref()).ok();
                        let _ = tx.send((id, thumbnail));
                    });
                }
            }
            model.thumbnail = thumbnails.get(&model.id).cloned().flatten();
        }
        // Asset discovery can complete after the transport handshake. Treat a
        // changed descriptor list as a real session revision so clients receive
        // a delta instead of remaining pinned to the initial two model entries.
        let model_fingerprint =
            serde_json::to_value(&models).expect("serializable model descriptors");
        let revision = {
            let mut session = self.session.lock().unwrap();
            if session.model_fingerprint.as_ref() != Some(&model_fingerprint) {
                if session.model_fingerprint.is_some() {
                    session.revision += 1;
                }
                session.model_fingerprint = Some(model_fingerprint);
            }
            session.revision
        };
        PlaygroundSnapshot {
            revision,
            values: BTreeMap::from([
                (
                    "npr".into(),
                    serde_json::to_value(&snapshot).expect("validated NPR state"),
                ),
                (
                    "models".into(),
                    serde_json::to_value(models).expect("model descriptors"),
                ),
                (
                    "look_previews".into(),
                    serde_json::to_value(look_previews).expect("look preview descriptors"),
                ),
                (
                    "look_preview_errors".into(),
                    serde_json::to_value(look_preview_errors).expect("look preview errors"),
                ),
                (
                    "brush_previews".into(),
                    serde_json::to_value(brush_preview_values).expect("brush preview descriptors"),
                ),
                (
                    "brush_preview_errors".into(),
                    serde_json::to_value(brush_preview_errors).expect("brush preview errors"),
                ),
                ("catalogs".into(), json!(*self.catalogs.lock().unwrap())),
                (
                    "transfer".into(),
                    json!(!self.imports.lock().unwrap().is_empty()),
                ),
                (
                    "diagnostics".into(),
                    json!(*self.state.render_stats.lock().unwrap()),
                ),
            ]),
            metadata: snapshot.metadata,
        }
    }
    fn dispatch(&self, action: &PlaygroundActionEnvelope) -> Result<(), PlaygroundActionError> {
        let intent =
            serde_json::from_value(action.intent.clone()).map_err(|e| PlaygroundActionError {
                control: action.control.clone(),
                code: "invalid_intent".into(),
                message: e.to_string(),
            })?;
        self.dispatch_intent(
            action.request_id,
            action.base_revision,
            action.control.clone(),
            intent,
        )
        .map(|_| ())
    }
    fn drain_events(&self) -> Vec<PlaygroundEvent> {
        std::mem::take(&mut self.session.lock().unwrap().events)
    }
}
