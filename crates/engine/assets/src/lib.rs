use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Mutex;

use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetKey(String);

impl AssetKey {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSourceKind {
    Engine,
    Mod(String),
    FileSystemRoot(String),
    Generated,
}

impl AssetSourceKind {
    pub fn label(&self) -> String {
        match self {
            Self::Engine => "engine".to_owned(),
            Self::Mod(mod_id) => format!("mod:{mod_id}"),
            Self::FileSystemRoot(root) => format!("fs:{root}"),
            Self::Generated => "generated".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AssetLoadPriority {
    Background,
    #[default]
    Interactive,
    Immediate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetManifest {
    pub key: AssetKey,
    pub source: AssetSourceKind,
    pub tags: Vec<String>,
}

impl AssetManifest {
    pub fn engine(key: AssetKey) -> Self {
        Self {
            key,
            source: AssetSourceKind::Engine,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetLoadRequest {
    pub key: AssetKey,
    pub priority: AssetLoadPriority,
}

impl AssetLoadRequest {
    pub fn new(key: AssetKey, priority: AssetLoadPriority) -> Self {
        Self { key, priority }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetEvent {
    ManifestRegistered(AssetKey),
    LoadRequested(AssetLoadRequest),
    ReloadRequested(AssetLoadRequest),
    LoadCompleted(AssetKey),
    LoadFailed { key: AssetKey, reason: String },
    Prepared(AssetKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedAsset {
    pub key: AssetKey,
    pub source: AssetSourceKind,
    pub resolved_path: PathBuf,
    pub byte_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedAsset {
    pub key: AssetKey,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedAssetKind {
    Image2d,
    Sprite2d,
    SpriteSheet2d,
    TileSet2d,
    TileRuleSet2d,
    Font2d,
    Font3d,
    Mesh3d,
    Material3d,
    GeneratedAudio,
    Unknown(String),
}

impl PreparedAssetKind {
    pub fn from_placeholder_kind(value: &str) -> Self {
        match value {
            "image-2d" => Self::Image2d,
            "sprite-2d" => Self::Sprite2d,
            "sprite-sheet-2d" => Self::SpriteSheet2d,
            "tileset-2d" => Self::TileSet2d,
            "tile-ruleset-2d" => Self::TileRuleSet2d,
            "font-2d" => Self::Font2d,
            "font-3d" => Self::Font3d,
            "mesh-3d" => Self::Mesh3d,
            "material-3d" => Self::Material3d,
            "generated-audio" => Self::GeneratedAudio,
            other => Self::Unknown(other.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Image2d => "image-2d",
            Self::Sprite2d => "sprite-2d",
            Self::SpriteSheet2d => "sprite-sheet-2d",
            Self::TileSet2d => "tileset-2d",
            Self::TileRuleSet2d => "tile-ruleset-2d",
            Self::Font2d => "font-2d",
            Self::Font3d => "font-3d",
            Self::Mesh3d => "mesh-3d",
            Self::Material3d => "material-3d",
            Self::GeneratedAudio => "generated-audio",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAsset {
    pub key: AssetKey,
    pub source: AssetSourceKind,
    pub resolved_path: PathBuf,
    pub byte_len: u64,
    pub kind: PreparedAssetKind,
    pub label: Option<String>,
    pub format: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Default)]
struct AssetCatalogState {
    manifests: BTreeMap<AssetKey, AssetManifest>,
    pending_loads: BTreeMap<AssetKey, AssetLoadRequest>,
    loaded_assets: BTreeMap<AssetKey, LoadedAsset>,
    prepared_assets: BTreeMap<AssetKey, PreparedAsset>,
    failed_assets: BTreeMap<AssetKey, FailedAsset>,
    events: Vec<AssetEvent>,
}

#[derive(Debug, Default)]
pub struct AssetCatalog {
    state: Mutex<AssetCatalogState>,
}

impl AssetCatalog {
    pub fn register(&self, key: AssetKey) -> bool {
        self.register_manifest(AssetManifest::engine(key))
    }

    pub fn register_manifest(&self, manifest: AssetManifest) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let key = manifest.key.clone();
        let inserted = state.manifests.insert(key.clone(), manifest).is_none();

        if inserted {
            state.events.push(AssetEvent::ManifestRegistered(key));
        }

        inserted
    }

    pub fn request_load(&self, request: AssetLoadRequest) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        if state.loaded_assets.contains_key(&request.key) {
            return;
        }
        match state.pending_loads.get_mut(&request.key) {
            Some(existing) if request.priority > existing.priority => {
                existing.priority = request.priority;
            }
            Some(_) => {}
            None => {
                state
                    .pending_loads
                    .insert(request.key.clone(), request.clone());
            }
        }
        state.events.push(AssetEvent::LoadRequested(request));
    }

    pub fn request_reload(&self, request: AssetLoadRequest) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.remove(&request.key);
        state.prepared_assets.remove(&request.key);
        state.failed_assets.remove(&request.key);
        match state.pending_loads.get_mut(&request.key) {
            Some(existing) if request.priority > existing.priority => {
                existing.priority = request.priority;
            }
            Some(_) => {}
            None => {
                state
                    .pending_loads
                    .insert(request.key.clone(), request.clone());
            }
        }
        state.events.push(AssetEvent::ReloadRequested(request));
    }

    pub fn mark_loaded(&self, asset: LoadedAsset) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let key = asset.key.clone();
        state.pending_loads.remove(&key);
        state.failed_assets.remove(&key);
        state.loaded_assets.insert(key.clone(), asset);
        state.events.push(AssetEvent::LoadCompleted(key));
    }

    pub fn mark_failed(&self, key: AssetKey, reason: impl Into<String>) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let reason = reason.into();
        state.pending_loads.remove(&key);
        state.loaded_assets.remove(&key);
        state.prepared_assets.remove(&key);
        state.failed_assets.insert(
            key.clone(),
            FailedAsset {
                key: key.clone(),
                reason: reason.clone(),
            },
        );
        state.events.push(AssetEvent::LoadFailed { key, reason });
    }

    pub fn mark_prepared(&self, asset: PreparedAsset) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let key = asset.key.clone();
        state.failed_assets.remove(&key);
        state.prepared_assets.insert(key.clone(), asset);
        state.events.push(AssetEvent::Prepared(key));
    }

    pub fn contains(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.contains_key(key)
    }

    pub fn manifest(&self, key: &AssetKey) -> Option<AssetManifest> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.get(key).cloned()
    }

    pub fn registered_keys(&self) -> Vec<AssetKey> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.keys().cloned().collect()
    }

    pub fn manifests(&self) -> Vec<AssetManifest> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.values().cloned().collect()
    }

    pub fn manifests_for_mod(&self, mod_id: &str) -> Vec<AssetManifest> {
        self.manifests()
            .into_iter()
            .filter(|manifest| matches!(&manifest.source, AssetSourceKind::Mod(source_mod) if source_mod == mod_id))
            .collect()
    }

    pub fn manifests_with_tag(&self, tag: &str) -> Vec<AssetManifest> {
        self.manifests()
            .into_iter()
            .filter(|manifest| manifest.tags.iter().any(|entry| entry == tag))
            .collect()
    }

    pub fn tags_for(&self, key: &AssetKey) -> Vec<String> {
        self.manifest(key)
            .map(|manifest| manifest.tags)
            .unwrap_or_default()
    }

    pub fn loaded_asset(&self, key: &AssetKey) -> Option<LoadedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.get(key).cloned()
    }

    pub fn failed_asset(&self, key: &AssetKey) -> Option<FailedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.failed_assets.get(key).cloned()
    }

    pub fn loaded_assets(&self) -> Vec<LoadedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.values().cloned().collect()
    }

    pub fn prepared_asset(&self, key: &AssetKey) -> Option<PreparedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.prepared_assets.get(key).cloned()
    }

    pub fn prepared_assets(&self) -> Vec<PreparedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.prepared_assets.values().cloned().collect()
    }

    pub fn failed_assets(&self) -> Vec<FailedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.failed_assets.values().cloned().collect()
    }

    pub fn is_loaded(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.contains_key(key)
    }

    pub fn is_prepared(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.prepared_assets.contains_key(key)
    }

    pub fn is_failed(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.failed_assets.contains_key(key)
    }

    pub fn pending_loads(&self) -> Vec<AssetLoadRequest> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let mut requests = state.pending_loads.values().cloned().collect::<Vec<_>>();
        requests.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.key.as_str().cmp(right.key.as_str()))
        });
        requests
    }

    pub fn drain_pending_loads(&self) -> Vec<AssetLoadRequest> {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let drained = state.pending_loads.values().cloned().collect::<Vec<_>>();
        state.pending_loads.clear();

        let mut drained = drained;
        drained.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.key.as_str().cmp(right.key.as_str()))
        });
        drained
    }

    pub fn drain_events(&self) -> Vec<AssetEvent> {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.events.drain(..).collect()
    }
}

pub fn prepare_debug_placeholder_asset(
    loaded_asset: &LoadedAsset,
    contents: &str,
) -> Result<PreparedAsset, String> {
    let mut metadata = BTreeMap::new();

    for (line_index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            return Err(format!(
                "invalid placeholder asset line {} in `{}`: expected `key = value`",
                line_index + 1,
                loaded_asset.resolved_path.display()
            ));
        };
        let key = raw_key.trim();
        if key.is_empty() {
            return Err(format!(
                "invalid placeholder asset line {} in `{}`: empty metadata key",
                line_index + 1,
                loaded_asset.resolved_path.display()
            ));
        }

        let value = parse_placeholder_value(raw_value.trim(), loaded_asset)?;
        metadata.insert(key.to_owned(), value);
    }

    let kind = metadata.get("kind").cloned().ok_or_else(|| {
        format!(
            "placeholder asset `{}` is missing `kind`",
            loaded_asset.key.as_str()
        )
    })?;
    let label = metadata.get("label").cloned();
    let format = metadata.get("format").cloned();

    Ok(PreparedAsset {
        key: loaded_asset.key.clone(),
        source: loaded_asset.source.clone(),
        resolved_path: loaded_asset.resolved_path.clone(),
        byte_len: loaded_asset.byte_len,
        kind: PreparedAssetKind::from_placeholder_kind(&kind),
        label,
        format,
        metadata,
    })
}

pub fn prepare_asset_from_contents(
    loaded_asset: &LoadedAsset,
    contents: &str,
) -> Result<PreparedAsset, String> {
    match loaded_asset
        .resolved_path
        .extension()
        .and_then(OsStr::to_str)
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("yml") | Some("yaml") => prepare_yaml_asset(loaded_asset, contents),
        Some("toml") => prepare_toml_asset(loaded_asset, contents),
        _ => prepare_debug_placeholder_asset(loaded_asset, contents),
    }
}

fn prepare_yaml_asset(loaded_asset: &LoadedAsset, contents: &str) -> Result<PreparedAsset, String> {
    let value = serde_yaml::from_str::<YamlValue>(contents).map_err(|error| {
        format!(
            "invalid yaml asset metadata in `{}`: {error}",
            loaded_asset.resolved_path.display()
        )
    })?;
    prepare_structured_asset(loaded_asset, flatten_yaml_value(&value))
}

fn prepare_toml_asset(loaded_asset: &LoadedAsset, contents: &str) -> Result<PreparedAsset, String> {
    let value = toml::from_str::<TomlValue>(contents).map_err(|error| {
        format!(
            "invalid toml asset metadata in `{}`: {error}",
            loaded_asset.resolved_path.display()
        )
    })?;
    prepare_structured_asset(loaded_asset, flatten_toml_value(&value))
}

fn prepare_structured_asset(
    loaded_asset: &LoadedAsset,
    metadata: BTreeMap<String, String>,
) -> Result<PreparedAsset, String> {
    let kind = metadata.get("kind").cloned().ok_or_else(|| {
        format!(
            "asset metadata `{}` is missing `kind`",
            loaded_asset.key.as_str()
        )
    })?;
    let label = metadata.get("label").cloned();
    let format = metadata.get("format").cloned();

    Ok(PreparedAsset {
        key: loaded_asset.key.clone(),
        source: loaded_asset.source.clone(),
        resolved_path: loaded_asset.resolved_path.clone(),
        byte_len: loaded_asset.byte_len,
        kind: PreparedAssetKind::from_placeholder_kind(&kind),
        label,
        format,
        metadata,
    })
}

fn flatten_yaml_value(value: &YamlValue) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    flatten_yaml_value_into(None, value, &mut metadata);
    metadata
}

fn flatten_yaml_value_into(
    prefix: Option<&str>,
    value: &YamlValue,
    metadata: &mut BTreeMap<String, String>,
) {
    match value {
        YamlValue::Mapping(mapping) => {
            for (key, value) in mapping {
                let key = match key {
                    YamlValue::String(value) => value.as_str().to_owned(),
                    other => stringify_yaml_scalar(other),
                };
                let next_prefix = match prefix {
                    Some(prefix) => format!("{prefix}.{key}"),
                    None => key,
                };
                flatten_yaml_value_into(Some(next_prefix.as_str()), value, metadata);
            }
        }
        YamlValue::Sequence(sequence) => {
            if let Some(prefix) = prefix {
                metadata.insert(
                    prefix.to_owned(),
                    sequence
                        .iter()
                        .map(stringify_yaml_scalar)
                        .collect::<Vec<_>>()
                        .join(","),
                );

                for (index, value) in sequence.iter().enumerate() {
                    let indexed_prefix = format!("{prefix}.{index}");
                    match value {
                        YamlValue::Mapping(_) | YamlValue::Sequence(_) => {
                            flatten_yaml_value_into(Some(indexed_prefix.as_str()), value, metadata);
                        }
                        other => {
                            metadata.insert(indexed_prefix, stringify_yaml_scalar(other));
                        }
                    }
                }
            }
        }
        other => {
            if let Some(prefix) = prefix {
                metadata.insert(prefix.to_owned(), stringify_yaml_scalar(other));
            }
        }
    }
}

fn stringify_yaml_scalar(value: &YamlValue) -> String {
    match value {
        YamlValue::Null => "null".to_owned(),
        YamlValue::Bool(value) => value.to_string(),
        YamlValue::Number(value) => value.to_string(),
        YamlValue::String(value) => value.clone(),
        YamlValue::Sequence(sequence) => sequence
            .iter()
            .map(stringify_yaml_scalar)
            .collect::<Vec<_>>()
            .join(","),
        YamlValue::Mapping(_) => "<mapping>".to_owned(),
        YamlValue::Tagged(value) => stringify_yaml_scalar(&value.value),
    }
}

fn flatten_toml_value(value: &TomlValue) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    flatten_toml_value_into(None, value, &mut metadata);
    metadata
}

fn flatten_toml_value_into(
    prefix: Option<&str>,
    value: &TomlValue,
    metadata: &mut BTreeMap<String, String>,
) {
    match value {
        TomlValue::Table(table) => {
            for (key, value) in table {
                let next_prefix = match prefix {
                    Some(prefix) => format!("{prefix}.{key}"),
                    None => key.clone(),
                };
                flatten_toml_value_into(Some(next_prefix.as_str()), value, metadata);
            }
        }
        TomlValue::Array(array) => {
            if let Some(prefix) = prefix {
                metadata.insert(
                    prefix.to_owned(),
                    array
                        .iter()
                        .map(stringify_toml_scalar)
                        .collect::<Vec<_>>()
                        .join(","),
                );

                for (index, value) in array.iter().enumerate() {
                    let indexed_prefix = format!("{prefix}.{index}");
                    match value {
                        TomlValue::Table(_) | TomlValue::Array(_) => {
                            flatten_toml_value_into(Some(indexed_prefix.as_str()), value, metadata);
                        }
                        other => {
                            metadata.insert(indexed_prefix, stringify_toml_scalar(other));
                        }
                    }
                }
            }
        }
        other => {
            if let Some(prefix) = prefix {
                metadata.insert(prefix.to_owned(), stringify_toml_scalar(other));
            }
        }
    }
}

fn stringify_toml_scalar(value: &TomlValue) -> String {
    match value {
        TomlValue::String(value) => value.clone(),
        TomlValue::Integer(value) => value.to_string(),
        TomlValue::Float(value) => value.to_string(),
        TomlValue::Boolean(value) => value.to_string(),
        TomlValue::Datetime(value) => value.to_string(),
        TomlValue::Array(values) => values
            .iter()
            .map(stringify_toml_scalar)
            .collect::<Vec<_>>()
            .join(","),
        TomlValue::Table(_) => "<table>".to_owned(),
    }
}

fn parse_placeholder_value(value: &str, loaded_asset: &LoadedAsset) -> Result<String, String> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return Ok(value[1..value.len() - 1].to_owned());
    }

    if value.contains('"') {
        return Err(format!(
            "invalid quoted placeholder value in `{}`",
            loaded_asset.resolved_path.display()
        ));
    }

    Ok(value.to_owned())
}

pub struct AssetsPlugin;

impl RuntimePlugin for AssetsPlugin {
    fn name(&self) -> &'static str {
        "amigo-assets"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(AssetCatalog::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AssetCatalog, AssetEvent, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest,
        AssetSourceKind, LoadedAsset, PreparedAsset, PreparedAssetKind,
        prepare_asset_from_contents, prepare_debug_placeholder_asset,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn registers_manifest_and_requests_load() {
        let catalog = AssetCatalog::default();
        let key = AssetKey::new("mods/core/textures/logo.png");

        let inserted = catalog.register_manifest(AssetManifest {
            key: key.clone(),
            source: AssetSourceKind::Mod("core".to_owned()),
            tags: vec!["ui".to_owned(), "logo".to_owned()],
        });
        catalog.request_load(AssetLoadRequest::new(
            key.clone(),
            AssetLoadPriority::Immediate,
        ));

        assert!(inserted);
        assert!(catalog.contains(&key));
        assert_eq!(
            catalog.manifest(&key).expect("manifest should exist").tags,
            vec!["ui".to_owned(), "logo".to_owned()]
        );
        assert_eq!(catalog.pending_loads().len(), 1);

        let events = catalog.drain_events();
        assert_eq!(
            events,
            vec![
                AssetEvent::ManifestRegistered(key.clone()),
                AssetEvent::LoadRequested(AssetLoadRequest::new(
                    key.clone(),
                    AssetLoadPriority::Immediate,
                )),
            ]
        );
        assert_eq!(catalog.drain_pending_loads().len(), 1);
    }

    #[test]
    fn coalesces_pending_loads_to_highest_priority() {
        let catalog = AssetCatalog::default();
        let key = AssetKey::new("mods/playground-3d/meshes/probe.mesh");

        catalog.request_load(AssetLoadRequest::new(
            key.clone(),
            AssetLoadPriority::Background,
        ));
        catalog.request_load(AssetLoadRequest::new(
            key.clone(),
            AssetLoadPriority::Immediate,
        ));
        catalog.request_load(AssetLoadRequest::new(
            key.clone(),
            AssetLoadPriority::Interactive,
        ));

        assert_eq!(
            catalog.pending_loads(),
            vec![AssetLoadRequest::new(key, AssetLoadPriority::Immediate)]
        );
    }

    #[test]
    fn filters_manifests_by_mod_and_tag() {
        let catalog = AssetCatalog::default();
        let sprite_key = AssetKey::new("playground-2d/textures/sprite-lab");
        let mesh_key = AssetKey::new("playground-3d/meshes/probe");

        catalog.register_manifest(AssetManifest {
            key: sprite_key.clone(),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            tags: vec!["phase3".to_owned(), "2d".to_owned(), "sprite".to_owned()],
        });
        catalog.register_manifest(AssetManifest {
            key: mesh_key.clone(),
            source: AssetSourceKind::Mod("playground-3d".to_owned()),
            tags: vec!["phase3".to_owned(), "3d".to_owned(), "mesh".to_owned()],
        });

        assert_eq!(catalog.manifests_for_mod("playground-2d").len(), 1);
        assert_eq!(catalog.manifests_with_tag("mesh").len(), 1);
        assert_eq!(
            catalog.tags_for(&mesh_key),
            vec!["phase3".to_owned(), "3d".to_owned(), "mesh".to_owned()]
        );
        assert_eq!(
            catalog
                .manifest(&sprite_key)
                .expect("sprite manifest should exist")
                .source
                .label(),
            "mod:playground-2d"
        );
    }

    #[test]
    fn tracks_loaded_and_failed_asset_states() {
        let catalog = AssetCatalog::default();
        let loaded_key = AssetKey::new("playground-2d/textures/sprite-lab");
        let failed_key = AssetKey::new("playground-3d/materials/missing");

        catalog.register_manifest(AssetManifest {
            key: loaded_key.clone(),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            tags: vec!["2d".to_owned()],
        });
        catalog.register_manifest(AssetManifest {
            key: failed_key.clone(),
            source: AssetSourceKind::Mod("playground-3d".to_owned()),
            tags: vec!["3d".to_owned()],
        });
        catalog.request_load(AssetLoadRequest::new(
            loaded_key.clone(),
            AssetLoadPriority::Immediate,
        ));
        catalog.request_load(AssetLoadRequest::new(
            failed_key.clone(),
            AssetLoadPriority::Interactive,
        ));

        catalog.mark_loaded(LoadedAsset {
            key: loaded_key.clone(),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            resolved_path: PathBuf::from("mods/playground-2d/textures/sprite-lab"),
            byte_len: 42,
        });
        catalog.mark_failed(failed_key.clone(), "file not found");

        assert!(catalog.is_loaded(&loaded_key));
        assert!(catalog.is_failed(&failed_key));
        assert_eq!(catalog.pending_loads().len(), 0);
        assert_eq!(
            catalog
                .loaded_asset(&loaded_key)
                .expect("loaded asset should exist")
                .byte_len,
            42
        );
        assert_eq!(
            catalog
                .failed_asset(&failed_key)
                .expect("failed asset should exist")
                .reason,
            "file not found"
        );
        assert_eq!(catalog.loaded_assets().len(), 1);
        assert_eq!(catalog.prepared_assets().len(), 0);
        assert_eq!(catalog.failed_assets().len(), 1);
    }

    #[test]
    fn tracks_prepared_asset_states() {
        let catalog = AssetCatalog::default();
        let key = AssetKey::new("playground-2d/textures/sprite-lab");

        catalog.mark_prepared(PreparedAsset {
            key: key.clone(),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            resolved_path: PathBuf::from("mods/playground-2d/textures/sprite-lab"),
            byte_len: 84,
            kind: PreparedAssetKind::Sprite2d,
            label: Some("Sprite Lab Placeholder".to_owned()),
            format: Some("debug-placeholder".to_owned()),
            metadata: BTreeMap::from([
                ("kind".to_owned(), "sprite-2d".to_owned()),
                ("label".to_owned(), "Sprite Lab Placeholder".to_owned()),
                ("format".to_owned(), "debug-placeholder".to_owned()),
            ]),
        });

        assert!(catalog.is_prepared(&key));
        assert_eq!(catalog.prepared_assets().len(), 1);
        assert_eq!(
            catalog
                .prepared_asset(&key)
                .expect("prepared asset should exist")
                .kind
                .as_str(),
            "sprite-2d"
        );
    }

    #[test]
    fn request_reload_requeues_loaded_and_prepared_asset() {
        let catalog = AssetCatalog::default();
        let key = AssetKey::new("playground-2d/textures/sprite-lab");

        catalog.mark_loaded(LoadedAsset {
            key: key.clone(),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            resolved_path: PathBuf::from("mods/playground-2d/textures/sprite-lab"),
            byte_len: 84,
        });
        catalog.mark_prepared(PreparedAsset {
            key: key.clone(),
            source: AssetSourceKind::Mod("playground-2d".to_owned()),
            resolved_path: PathBuf::from("mods/playground-2d/textures/sprite-lab"),
            byte_len: 84,
            kind: PreparedAssetKind::Sprite2d,
            label: Some("Sprite Lab Placeholder".to_owned()),
            format: Some("debug-placeholder".to_owned()),
            metadata: BTreeMap::from([
                ("kind".to_owned(), "sprite-2d".to_owned()),
                ("label".to_owned(), "Sprite Lab Placeholder".to_owned()),
                ("format".to_owned(), "debug-placeholder".to_owned()),
            ]),
        });

        catalog.request_reload(AssetLoadRequest::new(
            key.clone(),
            AssetLoadPriority::Immediate,
        ));

        assert!(!catalog.is_loaded(&key));
        assert!(!catalog.is_prepared(&key));
        assert_eq!(
            catalog.pending_loads(),
            vec![AssetLoadRequest::new(
                key.clone(),
                AssetLoadPriority::Immediate
            )]
        );
        assert!(catalog.drain_events().iter().any(
            |event| matches!(event, AssetEvent::ReloadRequested(request) if request.key == key)
        ));
    }

    #[test]
    fn parses_debug_placeholder_asset_metadata() {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-3d/materials/debug-surface"),
            source: AssetSourceKind::Mod("playground-3d".to_owned()),
            resolved_path: PathBuf::from("mods/playground-3d/materials/debug-surface"),
            byte_len: 96,
        };

        let prepared = prepare_debug_placeholder_asset(
            &loaded,
            r#"
                kind = "material-3d"
                label = "Debug Surface Placeholder"
                format = "debug-placeholder"
            "#,
        )
        .expect("placeholder asset should parse");

        assert_eq!(prepared.kind, PreparedAssetKind::Material3d);
        assert_eq!(prepared.label.as_deref(), Some("Debug Surface Placeholder"));
        assert_eq!(prepared.format.as_deref(), Some("debug-placeholder"));
        assert_eq!(
            prepared.metadata.get("kind").map(String::as_str),
            Some("material-3d")
        );
    }

    #[test]
    fn parses_yaml_sprite_sheet_asset_metadata() {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/textures/player"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/textures/player.yml"),
            byte_len: 128,
        };

        let prepared = prepare_asset_from_contents(
            &loaded,
            r#"
kind: sprite-sheet-2d
image: player.png
label: Sidescroller Player
frame_size:
  x: 32
  y: 32
columns: 8
rows: 4
animations:
  idle:
    frames: [0, 1, 2, 3]
    fps: 6
    looping: true
"#,
        )
        .expect("yaml sprite sheet metadata should parse");

        assert_eq!(prepared.kind, PreparedAssetKind::SpriteSheet2d);
        assert_eq!(prepared.label.as_deref(), Some("Sidescroller Player"));
        assert_eq!(
            prepared.metadata.get("image").map(String::as_str),
            Some("player.png")
        );
        assert_eq!(
            prepared.metadata.get("frame_size.x").map(String::as_str),
            Some("32")
        );
        assert_eq!(
            prepared
                .metadata
                .get("animations.idle.frames")
                .map(String::as_str),
            Some("0,1,2,3")
        );
    }

    #[test]
    fn parses_yaml_generated_audio_metadata() {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/audio/jump"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/audio/jump.yml"),
            byte_len: 96,
        };

        let prepared = prepare_asset_from_contents(
            &loaded,
            r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 44100
sequence:
  - wave: square
    frequency: 330
    duration_ms: 40
    volume: 0.35
envelope:
  attack_ms: 2
  release_ms: 30
"#,
        )
        .expect("yaml generated audio metadata should parse");

        assert_eq!(prepared.kind, PreparedAssetKind::GeneratedAudio);
        assert_eq!(
            prepared.metadata.get("generator").map(String::as_str),
            Some("pc-speaker")
        );
        assert_eq!(
            prepared.metadata.get("sample_rate").map(String::as_str),
            Some("44100")
        );
        assert_eq!(
            prepared.metadata.get("sequence").map(String::as_str),
            Some("<mapping>")
        );
        assert_eq!(
            prepared.metadata.get("sequence.0.wave").map(String::as_str),
            Some("square")
        );
        assert_eq!(
            prepared
                .metadata
                .get("sequence.0.frequency")
                .map(String::as_str),
            Some("330")
        );
        assert_eq!(
            prepared
                .metadata
                .get("envelope.attack_ms")
                .map(String::as_str),
            Some("2")
        );
    }

    #[test]
    fn parses_yaml_tile_ruleset_asset_metadata() {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/tilesets/platformer-rules"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from(
                "mods/playground-sidescroller/tilesets/platformer-rules.yml",
            ),
            byte_len: 256,
        };

        let prepared = prepare_asset_from_contents(
            &loaded,
            r##"
kind: tile-ruleset-2d
label: Platformer Ground Rules
format: amigo-rules-v1
tile_size:
  x: 16
  y: 16
terrains:
  ground:
    symbol: "#"
    collision: solid
    variants:
      single: 1
      left_cap: 2
      middle: 3
      right_cap: 4
      top_cap: 5
      bottom_cap: 6
"##,
        )
        .expect("yaml tile ruleset metadata should parse");

        assert_eq!(prepared.kind, PreparedAssetKind::TileRuleSet2d);
        assert_eq!(prepared.label.as_deref(), Some("Platformer Ground Rules"));
        assert_eq!(prepared.format.as_deref(), Some("amigo-rules-v1"));
        assert_eq!(
            prepared.metadata.get("tile_size.x").map(String::as_str),
            Some("16")
        );
        assert_eq!(
            prepared
                .metadata
                .get("terrains.ground.symbol")
                .map(String::as_str),
            Some("#")
        );
        assert_eq!(
            prepared
                .metadata
                .get("terrains.ground.variants.left_cap")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            prepared
                .metadata
                .get("terrains.ground.variants.top_cap")
                .map(String::as_str),
            Some("5")
        );
    }

    #[test]
    fn parses_toml_tile_ruleset_asset_metadata() {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/tilesets/platformer-rules"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from(
                "mods/playground-sidescroller/tilesets/platformer-rules.toml",
            ),
            byte_len: 256,
        };

        let prepared = prepare_asset_from_contents(
            &loaded,
            r##"kind = "tile-ruleset-2d"
label = "Platformer Ground Rules"
format = "amigo-rules-v1"

[tile_size]
x = 16
y = 16

[terrains.ground]
symbol = "#"
collision = "solid"

[terrains.ground.variants]
single = 1
left_cap = 2
middle = 3
right_cap = 4
top_cap = 5
"##,
        )
        .expect("toml tile ruleset metadata should parse");

        assert_eq!(prepared.kind, PreparedAssetKind::TileRuleSet2d);
        assert_eq!(
            prepared
                .metadata
                .get("terrains.ground.collision")
                .map(String::as_str),
            Some("solid")
        );
        assert_eq!(
            prepared
                .metadata
                .get("terrains.ground.variants.single")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            prepared
                .metadata
                .get("terrains.ground.variants.right_cap")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            prepared
                .metadata
                .get("terrains.ground.variants.top_cap")
                .map(String::as_str),
            Some("5")
        );
    }
}
