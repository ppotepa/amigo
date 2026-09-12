//! Authored NPR documents and deterministic inheritance, independent of UI hosts.
pub mod migration;
use amigo_render_npr::{BrushLibrary, ComicInk, NprStyleLayer, NprStyleLayers};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

use crate::state::{ObjectSettings, Settings};

/// Built-in brushes are immutable. Saving a modified one creates a new version
/// in the active mod; document references continue to point to their version.
pub fn builtin_brush_library() -> BrushLibrary {
    BrushLibrary::drawing_studio()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprLayerDocument {
    pub layer_id: String,
    pub order: i32,
    pub parameters: NprStyleLayer,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprLookPatch {
    /// Partial fields of typed ComicInk. Resolution validates the complete result.
    #[serde(default)]
    pub style: BTreeMap<String, Value>,
    #[serde(default)]
    pub layers: Vec<NprLayerDocument>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprLookDocument {
    pub id: String,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub look: NprLookPatch,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprModelOverrideDocument {
    #[serde(default)]
    pub look: NprLookPatch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprSceneProfileDocument {
    pub version: u32,
    #[serde(default = "render_all_objects")]
    pub render_all_objects: bool,
    pub active_look: Option<String>,
    #[serde(default)]
    pub look: NprLookPatch,
    #[serde(default)]
    pub camera: NprSceneCameraDocument,
    #[serde(default)]
    pub motion: NprSceneMotionDocument,
    #[serde(default)]
    pub objects: BTreeMap<String, ObjectSettings>,
    #[serde(default = "scene_seed")]
    pub seed: u64,
    #[serde(default)]
    pub object_overrides: BTreeMap<String, NprModelOverrideDocument>,
    #[serde(default)]
    pub variants: BTreeMap<String, NprLookPatch>,
    #[serde(default)]
    pub brushes: BrushLibrary,
}

/// A local, model-bound Drawing Studio checkpoint. Drafts are intentionally
/// separate from scene profiles so selecting another source model can never
/// leak its camera, palette, paper or layer stack into the new drawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprDrawingDraft {
    pub version: u32,
    pub source_model: String,
    pub settings: Settings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprSceneCameraDocument {
    pub target: glam::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov: f32,
}
impl Default for NprSceneCameraDocument {
    fn default() -> Self {
        Self {
            target: glam::Vec3::ZERO,
            yaw: 0.,
            pitch: 0.,
            distance: 14.,
            fov: 45.,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NprSceneMotionDocument {
    pub paused: bool,
    pub sketch_paused: bool,
    pub speed: f32,
    pub policy: amigo_render_npr::NprMotionPolicy,
}
impl Default for NprSceneMotionDocument {
    fn default() -> Self {
        Self {
            paused: false,
            sketch_paused: false,
            speed: 1.,
            policy: Default::default(),
        }
    }
}
fn scene_seed() -> u64 {
    0x4e5052
}
fn render_all_objects() -> bool {
    false
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprResolvedLook {
    pub style: ComicInk,
    pub layers: NprStyleLayers,
}

fn merge_value(target: &mut Value, incoming: &Value) {
    match (target, incoming) {
        (Value::Object(target), Value::Object(incoming)) => {
            for (key, value) in incoming {
                merge_value(target.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (target, incoming) => *target = incoming.clone(),
    }
}

impl NprLookPatch {
    pub fn merge(&mut self, patch: &Self) -> Result<(), String> {
        // Reject the entire patch before changing any inherited state.
        let known = serde_json::to_value(ComicInk::default()).map_err(|e| e.to_string())?;
        for key in patch.style.keys() {
            if known.get(key).is_none() {
                return Err(format!("unknown look field: {key}"));
            }
        }
        let mut seen = BTreeSet::new();
        for layer in &patch.layers {
            if layer.layer_id != layer.parameters.id || !seen.insert(&layer.layer_id) {
                return Err(format!("invalid or duplicate layer_id: {}", layer.layer_id));
            }
        }
        for (key, value) in &patch.style {
            merge_value(self.style.entry(key.clone()).or_insert(Value::Null), value);
        }
        {
            let target = &mut self.layers;
            let source = &patch.layers;
            for layer in source {
                if let Some(existing) = target.iter_mut().find(|p| p.layer_id == layer.layer_id) {
                    *existing = layer.clone();
                } else {
                    target.push(layer.clone());
                }
            }
            target.sort_by(|a, b| (a.order, &a.layer_id).cmp(&(b.order, &b.layer_id)));
        }
        Ok(())
    }

    pub fn from_resolved(resolved: &NprResolvedLook) -> Result<Self, String> {
        let style = serde_json::from_value(
            serde_json::to_value(resolved.style).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mut patch = Self {
            style,
            ..Self::default()
        };
        for (order, layer) in resolved.layers.layers.iter().enumerate() {
            patch.layers.push(NprLayerDocument {
                layer_id: layer.id.clone(),
                order: order as i32,
                parameters: layer.clone(),
            });
        }
        Ok(patch)
    }

    pub fn changes(before: &NprResolvedLook, after: &NprResolvedLook) -> Result<Self, String> {
        let before = Self::from_resolved(before)?;
        let mut after = Self::from_resolved(after)?;
        after
            .style
            .retain(|key, value| before.style.get(key) != Some(value));
        after.layers.retain(|layer| !before.layers.contains(layer));
        Ok(after)
    }
}

pub fn resolve_look(
    looks: &BTreeMap<String, NprLookDocument>,
    model: &NprLookPatch,
    active: Option<&str>,
    scene: &NprLookPatch,
    object: &NprLookPatch,
    live: &NprLookPatch,
) -> Result<NprResolvedLook, String> {
    fn visit(
        id: &str,
        looks: &BTreeMap<String, NprLookDocument>,
        stack: &mut Vec<String>,
        patch: &mut NprLookPatch,
    ) -> Result<(), String> {
        if stack.iter().any(|p| p == id) {
            return Err(format!(
                "look include cycle: {} -> {id}",
                stack.join(" -> ")
            ));
        }
        if stack.len() >= 64 {
            return Err("look include depth exceeds 64".into());
        }
        let look = looks.get(id).ok_or_else(|| format!("missing look: {id}"))?;
        if look.id != id {
            return Err(format!("look id does not match reference: {id}"));
        }
        stack.push(id.into());
        for include in &look.includes {
            visit(include, looks, stack, patch)?;
        }
        patch.merge(&look.look)?;
        stack.pop();
        Ok(())
    }
    let mut patch = NprLookPatch::from_resolved(&NprResolvedLook {
        style: ComicInk::default(),
        layers: NprStyleLayers::default(),
    })?;
    patch.merge(model)?;
    if let Some(id) = active {
        visit(id, looks, &mut Vec::new(), &mut patch)?;
    }
    for incoming in [scene, object, live] {
        patch.merge(incoming)?;
    }
    let style: ComicInk =
        serde_json::from_value(serde_json::to_value(patch.style).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let mut ordered = patch.layers;
    ordered.sort_by(|a, b| (a.order, &a.layer_id).cmp(&(b.order, &b.layer_id)));
    let layers = NprStyleLayers {
        layers: ordered.into_iter().map(|p| p.parameters).collect(),
    };
    layers.validate()?;
    let mut validation = Settings::empty_scene(true);
    validation.global = style;
    validation.style_layers = layers.clone();
    validation.validate()?;
    Ok(NprResolvedLook { style, layers })
}

impl NprSceneProfileDocument {
    pub fn from_settings(settings: &Settings, active_look: Option<String>) -> Result<Self, String> {
        settings.validate()?;
        Ok(Self {
            version: 1,
            // New Drawing Studio documents never carry the retired gallery
            // execution mode. Older profiles are rejected explicitly in
            // `resolve` rather than silently rendering extra subjects.
            render_all_objects: false,
            active_look,
            look: NprLookPatch::from_resolved(&NprResolvedLook {
                style: settings.global,
                layers: settings.style_layers.clone(),
            })?,
            camera: NprSceneCameraDocument {
                target: settings.camera_target,
                yaw: settings.camera_yaw,
                pitch: settings.camera_pitch,
                distance: settings.camera_distance,
                fov: settings.camera_fov,
            },
            motion: NprSceneMotionDocument {
                paused: settings.paused,
                sketch_paused: settings.sketch_paused,
                speed: settings.speed,
                policy: settings.motion,
            },
            objects: settings.objects.clone(),
            seed: settings.seed,
            object_overrides: BTreeMap::new(),
            variants: BTreeMap::new(),
            brushes: builtin_brush_library(),
        })
    }
    pub fn resolve(
        &self,
        looks: &BTreeMap<String, NprLookDocument>,
        model_defaults: &BTreeMap<String, NprLookPatch>,
        live: &NprLookPatch,
    ) -> Result<Settings, String> {
        if self.version != 1 {
            return Err("unsupported NPR scene profile version".into());
        }
        if self.render_all_objects {
            return Err(
                "legacy multi-model NPR profile: migrate it to a Drawing Studio source_model before opening"
                    .into(),
            );
        }
        let mut settings = Settings::empty_scene(self.render_all_objects);
        settings.objects = self.objects.clone();
        settings.selected = settings.objects.keys().next().cloned().unwrap_or_default();
        settings.seed = self.seed;
        settings.camera_target = self.camera.target;
        settings.camera_yaw = self.camera.yaw;
        settings.camera_pitch = self.camera.pitch;
        settings.camera_distance = self.camera.distance;
        settings.camera_fov = self.camera.fov;
        settings.paused = self.motion.paused;
        settings.sketch_paused = self.motion.sketch_paused;
        settings.speed = self.motion.speed;
        settings.motion = self.motion.policy;
        let empty = NprLookPatch::default();
        let global = resolve_look(
            looks,
            &empty,
            self.active_look.as_deref(),
            &self.look,
            &empty,
            live,
        )?;
        settings.global = global.style;
        settings.style_layers = global.layers;
        let brush_library = if self.brushes.brushes.is_empty() {
            builtin_brush_library()
        } else {
            self.brushes.clone()
        };
        settings.brushes = brush_library.clone();
        settings.style_layers.validate_brushes(&brush_library)?;
        for (id, object) in &mut settings.objects {
            let mut object_patch = NprLookPatch::default();
            let overrides =
                serde_json::to_value(&object.style_overrides).map_err(|e| e.to_string())?;
            if let Some(values) = overrides.as_object() {
                object_patch.style = values
                    .iter()
                    .filter(|(_, v)| !v.is_null())
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
            }
            if !object.style_layer_overrides.is_empty() {
                let layers = object
                    .style_layer_overrides
                    .resolve(&settings.style_layers)?;
                let patch = NprLookPatch::from_resolved(&NprResolvedLook {
                    style: settings.global,
                    layers,
                })?;
                object_patch.layers = patch.layers;
            }
            object_patch.merge(
                self.object_overrides
                    .get(id)
                    .map(|o| &o.look)
                    .unwrap_or(&empty),
            )?;
            let resolved = resolve_look(
                looks,
                model_defaults.get(&object.model).unwrap_or(&empty),
                self.active_look.as_deref(),
                &self.look,
                &object_patch,
                live,
            )?;
            apply_resolved_object(
                object,
                &resolved,
                &global.style,
                &settings.style_layers,
                &object_patch,
            )?;
        }
        settings.validate()?;
        Ok(settings)
    }
}

fn apply_resolved_object(
    object: &mut ObjectSettings,
    resolved: &NprResolvedLook,
    inherited_style: &ComicInk,
    inherited: &NprStyleLayers,
    authored: &NprLookPatch,
) -> Result<(), String> {
    let base = serde_json::to_value(inherited_style).map_err(|e| e.to_string())?;
    let mut values = serde_json::to_value(resolved.style).map_err(|e| e.to_string())?;
    values
        .as_object_mut()
        .ok_or("invalid style")?
        .retain(|key, value| authored.style.contains_key(key) || base.get(key) != Some(value));
    object.style_overrides = serde_json::from_value(values).map_err(|e| e.to_string())?;
    // Layer overrides retain the renderer's typed stable-id contract.
    object.style_layer_overrides =
        amigo_render_npr::NprStyleLayerOverrides::from_resolved(inherited, &resolved.layers)?;
    Ok(())
}

/// Restricts authored names to a single stable identifier, including on Windows.
pub fn validate_document_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 80
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("document id must contain 1..80 letters, digits, '-' or '_'".into());
    }
    let upper = id.to_ascii_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str())
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.as_bytes()[3].is_ascii_digit())
    {
        return Err("reserved document id".into());
    }
    Ok(())
}

pub fn authored_path(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("authored path must stay inside the active mod".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let target = root.join(relative);
    let mut ancestor = target.as_path();
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or("missing ancestor")?;
    }
    if !ancestor
        .canonicalize()
        .map_err(|e| e.to_string())?
        .starts_with(&root)
    {
        return Err("authored path escapes active mod".into());
    }
    Ok(target)
}

#[derive(Debug)]
pub struct NprTrackedDocument {
    pub path: PathBuf,
    baseline: Option<[u8; 32]>,
    pub pending: Vec<u8>,
    pub dirty: bool,
    pub writable: bool,
}

impl NprTrackedDocument {
    pub fn open(path: PathBuf, writable: bool) -> Result<Self, String> {
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        Ok(Self {
            path,
            baseline: Some(Sha256::digest(&bytes).into()),
            pending: bytes,
            dirty: false,
            writable,
        })
    }
    pub fn new(path: PathBuf, bytes: Vec<u8>) -> Self {
        Self {
            path,
            baseline: None,
            pending: bytes,
            dirty: true,
            writable: true,
        }
    }
    pub fn stage<T: Serialize>(&mut self, document: &T) -> Result<(), String> {
        self.pending = serde_yaml::to_string(document)
            .map_err(|e| e.to_string())?
            .into_bytes();
        self.dirty = self.baseline != Some(Sha256::digest(&self.pending).into());
        Ok(())
    }
    pub fn check_conflict(&self) -> Result<(), String> {
        let current = match fs::read(&self.path) {
            Ok(bytes) => Some(Sha256::digest(&bytes).into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.to_string()),
        };
        if current != self.baseline {
            return Err(format!(
                "external change: {}; Reload or Save As",
                self.path.display()
            ));
        }
        Ok(())
    }
    pub fn reload(&mut self) -> Result<(), String> {
        *self = Self::open(self.path.clone(), self.writable)?;
        Ok(())
    }
    pub fn save(&mut self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        if !self.writable {
            return Err("document is read-only; use Save As".into());
        }
        self.check_conflict()?;
        let parent = self.path.parent().ok_or("missing document directory")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary
            .write_all(&self.pending)
            .map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        self.check_conflict()?;
        if self.baseline.is_none() {
            temporary
                .persist_noclobber(&self.path)
                .map_err(|e| e.to_string())?;
        } else {
            temporary.persist(&self.path).map_err(|e| e.to_string())?;
        }
        self.baseline = Some(Sha256::digest(&self.pending).into());
        self.dirty = false;
        Ok(())
    }
}

pub fn save_all(documents: &mut [NprTrackedDocument]) -> Result<(), String> {
    for doc in documents.iter().filter(|d| d.dirty && d.writable) {
        doc.check_conflict()?;
    }
    for doc in documents.iter_mut().filter(|d| d.dirty && d.writable) {
        doc.save()?;
    }
    Ok(())
}
