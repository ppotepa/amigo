//! Explicit conversion of split authored stacks. Never called by document loading.
use super::{NprLayerDocument, NprLookPatch, NprSceneProfileDocument, NprTrackedDocument};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Prepared conversion with the original SHA baseline retained through Apply.
pub struct LayerStackMigration {
    document: NprTrackedDocument,
    original: Vec<u8>,
    pub changes: Vec<String>,
}

/// Explicitly converts a retired gallery profile into one Drawing Studio
/// document. The caller must name the source model: guessing would silently
/// change the authored subject. The original profile is backed up by `apply`.
pub struct DrawingStudioProfileMigration {
    document: NprTrackedDocument,
    original: Vec<u8>,
    pub source_model: String,
    pub removed_models: Vec<String>,
}

impl DrawingStudioProfileMigration {
    pub fn preview(path: PathBuf, source_model: &str) -> Result<Self, String> {
        let mut document = NprTrackedDocument::open(path, true)?;
        let original = document.pending.clone();
        let mut profile: NprSceneProfileDocument =
            serde_yaml::from_slice(&original).map_err(|e| e.to_string())?;
        if !profile.render_all_objects {
            return Err("profile is already a Drawing Studio document".into());
        }
        let selected = profile
            .objects
            .get(source_model)
            .cloned()
            .ok_or_else(|| format!("legacy profile has no model `{source_model}`"))?;
        let removed_models = profile
            .objects
            .keys()
            .filter(|id| id.as_str() != source_model)
            .cloned()
            .collect::<Vec<_>>();
        profile.render_all_objects = false;
        profile.objects.clear();
        profile.objects.insert(source_model.into(), selected);
        profile
            .object_overrides
            .retain(|id, _| id.as_str() == source_model);
        document.stage(&profile)?;
        Ok(Self {
            document,
            original,
            source_model: source_model.into(),
            removed_models,
        })
    }

    pub fn output(&self) -> &[u8] {
        &self.document.pending
    }

    pub fn apply(mut self, backup: &Path) -> Result<(), String> {
        self.document.check_conflict()?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(backup)
            .map_err(|e| format!("backup {}: {e}", backup.display()))?;
        file.write_all(&self.original)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        self.document.save()
    }
}

impl LayerStackMigration {
    pub fn preview(path: PathBuf) -> Result<Self, String> {
        let mut document = NprTrackedDocument::open(path, true)?;
        let original = document.pending.clone();
        let mut value: Value = serde_yaml::from_slice(&original).map_err(|e| e.to_string())?;
        let mut changes = Vec::new();
        // Only document-owned look patches, never nested medium parameters.
        if let Some(look) = value.get_mut("look") {
            convert_patch(look, "look", &mut changes)?;
        } else if value.get("style").is_some()
            || value.get("paint").is_some()
            || value.get("stroke").is_some()
            || value.get("layers").is_some()
        {
            convert_patch(&mut value, "look", &mut changes)?;
        }
        if let Some(overrides) = value.get_mut("object_overrides") {
            for (id, object) in overrides
                .as_object_mut()
                .ok_or("invalid object_overrides")?
            {
                if let Some(look) = object.get_mut("look") {
                    convert_patch(look, &format!("object_overrides.{id}.look"), &mut changes)?;
                }
            }
        }
        if !changes.is_empty() {
            document.stage(&value)?;
        }
        Ok(Self {
            document,
            original,
            changes,
        })
    }

    pub fn output(&self) -> &[u8] {
        &self.document.pending
    }

    /// Backup is exclusive and durable before the existing atomic Save runs.
    /// Failure leaves the authored document untouched (or a recoverable backup).
    pub fn apply(mut self, backup: &Path) -> Result<(), String> {
        if self.changes.is_empty() {
            return Ok(());
        }
        self.document.check_conflict()?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(backup)
            .map_err(|e| format!("backup {}: {e}", backup.display()))?;
        file.write_all(&self.original)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        self.document.save()
    }
}

fn convert_patch(value: &mut Value, path: &str, changes: &mut Vec<String>) -> Result<(), String> {
    let map = value
        .as_object_mut()
        .ok_or_else(|| format!("{path}: expected a look mapping"))?;
    if map.contains_key("paint") || map.contains_key("stroke") {
        if map.contains_key("layers") {
            return Err(format!("{path}: mixed split and unified layer stacks"));
        }
        let mut layers = Vec::<NprLayerDocument>::new();
        for key in ["paint", "stroke"] {
            if let Some(value) = map.remove(key) {
                layers.extend(
                    serde_json::from_value::<Vec<NprLayerDocument>>(value)
                        .map_err(|e| format!("{path}.{key}: {e}"))?,
                );
            }
        }
        layers.sort_by(|a, b| (a.order, &a.layer_id).cmp(&(b.order, &b.layer_id)));
        let patch = NprLookPatch {
            layers,
            ..Default::default()
        };
        NprLookPatch::default().merge(&patch)?;
        let count = patch.layers.len();
        map.insert(
            "layers".into(),
            serde_json::to_value(patch.layers).map_err(|e| e.to_string())?,
        );
        changes.push(format!(
            "{path}: paint + stroke -> layers ({count} entries; order and parameters preserved)"
        ));
    }
    migrate_layer_sources(map, path, changes)?;
    // Ensure unknown fields cannot be silently discarded by this conversion.
    serde_json::from_value::<NprLookPatch>(value.clone()).map_err(|e| format!("{path}: {e}"))?;
    Ok(())
}

fn migrate_layer_sources(
    patch: &mut serde_json::Map<String, Value>,
    path: &str,
    changes: &mut Vec<String>,
) -> Result<(), String> {
    let Some(layers) = patch.get_mut("layers") else {
        return Ok(());
    };
    let layers = layers
        .as_array_mut()
        .ok_or_else(|| format!("{path}.layers: expected an array"))?;
    let mut migrated = 0;
    for layer in layers {
        let parameters = layer
            .get_mut("parameters")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| format!("{path}.layers: expected parameters"))?;
        let Some(kind) = parameters.remove("kind") else {
            continue;
        };
        if parameters.contains_key("source") {
            return Err(format!(
                "{path}: layer contains both legacy kind and source"
            ));
        }
        let source = match kind.as_str() {
            Some("paper") => "paper",
            Some("underpainting") => "wash",
            Some("fill") => "flat-fill",
            Some("hatching") => "shadow-hatch",
            Some("contours") => "silhouette",
            Some("creases") => "creases",
            Some("form-lines") => "form-lines",
            Some("construction") => "construction",
            Some(other) => return Err(format!("{path}: unknown legacy layer kind `{other}`")),
            None => return Err(format!("{path}: legacy layer kind must be a string")),
        };
        parameters.insert("source".into(), Value::String(source.into()));
        migrated += 1;
    }
    if migrated > 0 {
        changes.push(format!("{path}: kind -> source ({migrated} layer entries)"));
    }
    Ok(())
}
