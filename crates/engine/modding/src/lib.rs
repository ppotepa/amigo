use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use serde::{Deserialize, Serialize};

pub const CORE_MOD_ID: &str = "core";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub scripting: Option<ModScriptingManifest>,
    #[serde(default)]
    pub scenes: Vec<ModSceneManifest>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModScriptMode {
    Disabled,
    #[default]
    Bootstrap,
    Persistent,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModScriptingManifest {
    #[serde(default)]
    pub mod_script: Option<String>,
    #[serde(default)]
    pub mod_script_mode: ModScriptMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModSceneManifest {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub document: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default = "default_launcher_visible")]
    pub launcher_visible: bool,
}

impl ModSceneManifest {
    pub fn is_launcher_visible(&self) -> bool {
        self.launcher_visible
    }

    pub fn root_path(&self, mod_root: &Path) -> PathBuf {
        let relative_root = if self.path.is_empty() {
            PathBuf::from("scenes").join(&self.id)
        } else {
            PathBuf::from(&self.path)
        };
        mod_root.join(relative_root)
    }

    pub fn document_path(&self, mod_root: &Path) -> PathBuf {
        let scene_root = self.root_path(mod_root);
        match self.document.as_deref() {
            Some(document) => self.resolve_scene_file_path(mod_root, &scene_root, document),
            None => scene_root.join("scene.yml"),
        }
    }

    pub fn script_path(&self, mod_root: &Path) -> PathBuf {
        let scene_root = self.root_path(mod_root);
        match self.script.as_deref() {
            Some(script) => self.resolve_scene_file_path(mod_root, &scene_root, script),
            None => scene_root.join("scene.rhai"),
        }
    }

    fn resolve_scene_file_path(
        &self,
        mod_root: &Path,
        scene_root: &Path,
        relative_path: &str,
    ) -> PathBuf {
        if self.path.is_empty() {
            mod_root.join(relative_path)
        } else {
            scene_root.join(relative_path)
        }
    }
}

#[derive(Debug, Default)]
pub struct ModCatalog {
    mods: Vec<DiscoveredMod>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredMod {
    pub manifest: ModManifest,
    pub root_path: PathBuf,
}

impl DiscoveredMod {
    pub fn mod_script_path(&self) -> Option<PathBuf> {
        self.manifest
            .scripting
            .as_ref()
            .and_then(|scripting| scripting.mod_script.as_ref())
            .map(|mod_script| self.root_path.join(mod_script))
    }

    pub fn scene_by_id(&self, scene_id: &str) -> Option<&ModSceneManifest> {
        self.manifest
            .scenes
            .iter()
            .find(|scene| scene.id == scene_id)
    }

    pub fn scene_root_path(&self, scene_id: &str) -> Option<PathBuf> {
        self.scene_by_id(scene_id)
            .map(|scene| scene.root_path(&self.root_path))
    }

    pub fn scene_document_path(&self, scene_id: &str) -> Option<PathBuf> {
        self.scene_by_id(scene_id)
            .map(|scene| scene.document_path(&self.root_path))
    }

    pub fn scene_script_path(&self, scene_id: &str) -> Option<PathBuf> {
        self.scene_by_id(scene_id)
            .map(|scene| scene.script_path(&self.root_path))
    }
}

impl ModCatalog {
    pub fn from_discovered_mods(mods: Vec<DiscoveredMod>) -> Self {
        Self { mods }
    }

    pub fn discover_unresolved(mods_root: &Path) -> AmigoResult<Vec<DiscoveredMod>> {
        let mut discovered = discover_mod_map(mods_root)?
            .into_values()
            .collect::<Vec<_>>();
        discovered.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
        Ok(discovered)
    }

    pub fn discover(mods_root: &Path) -> AmigoResult<Self> {
        let discovered = discover_mod_map(mods_root)?;
        let selected_mod_ids = discovered.keys().cloned().collect::<Vec<_>>();
        let mods = resolve_discovered_mods(&discovered, &selected_mod_ids)?;
        Ok(Self { mods })
    }

    pub fn discover_selected(mods_root: &Path, selected_mod_ids: &[String]) -> AmigoResult<Self> {
        let discovered = discover_mod_map(mods_root)?;
        let mods = resolve_discovered_mods(&discovered, selected_mod_ids)?;
        Ok(Self { mods })
    }

    pub fn mods(&self) -> &[DiscoveredMod] {
        &self.mods
    }

    pub fn manifests(&self) -> impl Iterator<Item = &ModManifest> {
        self.mods
            .iter()
            .map(|discovered_mod| &discovered_mod.manifest)
    }

    pub fn mod_ids(&self) -> Vec<&str> {
        self.mods
            .iter()
            .map(|discovered_mod| discovered_mod.manifest.id.as_str())
            .collect()
    }

    pub fn mod_by_id(&self, mod_id: &str) -> Option<&DiscoveredMod> {
        self.mods
            .iter()
            .find(|discovered_mod| discovered_mod.manifest.id == mod_id)
    }
}

pub fn requested_mods_for_root(root_mod_id: &str) -> Vec<String> {
    if root_mod_id == CORE_MOD_ID {
        vec![CORE_MOD_ID.to_owned()]
    } else {
        vec![CORE_MOD_ID.to_owned(), root_mod_id.to_owned()]
    }
}

fn discover_mod_map(mods_root: &Path) -> AmigoResult<BTreeMap<String, DiscoveredMod>> {
    if !mods_root.exists() {
        return Ok(BTreeMap::new());
    }

    let mut discovered = BTreeMap::new();

    for entry in fs::read_dir(mods_root)? {
        let entry = entry?;
        let mod_root = entry.path();

        if !mod_root.is_dir() {
            continue;
        }

        let manifest_path = mod_root.join("mod.toml");

        if !manifest_path.is_file() {
            continue;
        }

        let raw = fs::read_to_string(&manifest_path)?;
        let manifest = toml::from_str::<ModManifest>(&raw)
            .map_err(|error| AmigoError::Message(error.to_string()))?;
        let mod_id = manifest.id.clone();

        if discovered
            .insert(
                mod_id.clone(),
                DiscoveredMod {
                    manifest,
                    root_path: mod_root,
                },
            )
            .is_some()
        {
            return Err(AmigoError::Message(format!(
                "duplicate mod id `{mod_id}` found under `{}`",
                mods_root.display()
            )));
        }
    }

    Ok(discovered)
}

fn resolve_discovered_mods(
    discovered: &BTreeMap<String, DiscoveredMod>,
    selected_mod_ids: &[String],
) -> AmigoResult<Vec<DiscoveredMod>> {
    let mut ordered = Vec::new();
    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();

    for mod_id in selected_mod_ids {
        resolve_mod(
            mod_id,
            None,
            discovered,
            &mut visiting,
            &mut visited,
            &mut ordered,
        )?;
    }

    Ok(ordered)
}

fn resolve_mod(
    mod_id: &str,
    requested_by: Option<&str>,
    discovered: &BTreeMap<String, DiscoveredMod>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    ordered: &mut Vec<DiscoveredMod>,
) -> AmigoResult<()> {
    if visited.contains(mod_id) {
        return Ok(());
    }

    if !visiting.insert(mod_id.to_owned()) {
        return Err(AmigoError::Message(format!(
            "dependency cycle detected while resolving mod `{mod_id}`"
        )));
    }

    let discovered_mod = discovered.get(mod_id).ok_or_else(|| {
        let message = match requested_by {
            Some(parent_mod_id) => {
                format!("mod `{parent_mod_id}` depends on missing mod `{mod_id}`")
            }
            None => format!("configured mod `{mod_id}` was not found in the discovered catalog"),
        };
        AmigoError::Message(message)
    })?;

    for dependency_id in discovered_mod.manifest.dependencies.clone() {
        resolve_mod(
            &dependency_id,
            Some(mod_id),
            discovered,
            visiting,
            visited,
            ordered,
        )?;
    }

    visiting.remove(mod_id);

    if visited.insert(mod_id.to_owned()) {
        ordered.push(discovered_mod.clone());
    }

    Ok(())
}

pub struct ModdingPlugin {
    mods_root: PathBuf,
    selected_mod_ids: Option<Vec<String>>,
}

impl ModdingPlugin {
    pub fn new(mods_root: impl Into<PathBuf>) -> Self {
        Self {
            mods_root: mods_root.into(),
            selected_mod_ids: None,
        }
    }

    pub fn with_selected_mods(
        mods_root: impl Into<PathBuf>,
        selected_mod_ids: impl Into<Vec<String>>,
    ) -> Self {
        Self {
            mods_root: mods_root.into(),
            selected_mod_ids: Some(selected_mod_ids.into()),
        }
    }
}

impl RuntimePlugin for ModdingPlugin {
    fn name(&self) -> &'static str {
        "amigo-modding"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let catalog = match &self.selected_mod_ids {
            Some(selected_mod_ids) => {
                ModCatalog::discover_selected(&self.mods_root, selected_mod_ids)?
            }
            None => ModCatalog::discover(&self.mods_root)?,
        };
        registry.register(catalog)
    }
}

fn default_launcher_visible() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn discovered_mod(id: &str, dependencies: &[&str]) -> DiscoveredMod {
        DiscoveredMod {
            manifest: ModManifest {
                id: id.to_owned(),
                name: id.to_owned(),
                version: "0.1.0".to_owned(),
                description: None,
                authors: Vec::new(),
                dependencies: dependencies
                    .iter()
                    .map(|dependency| (*dependency).to_owned())
                    .collect(),
                capabilities: Vec::new(),
                scripting: None,
                scenes: Vec::new(),
            },
            root_path: PathBuf::from(format!("mods/{id}")),
        }
    }

    #[test]
    fn deserializes_scripting_section_and_scene_paths() {
        let manifest = toml::from_str::<ModManifest>(
            r#"
                id = "playground-2d"
                name = "Playground 2D"
                version = "0.1.0"

                [scripting]
                mod_script = "scripts/mod.rhai"
                mod_script_mode = "persistent"

                [[scenes]]
                id = "basic-scripting-demo"
                label = "Basic Scripting Demo"
                path = "scenes/basic-scripting-demo"
                launcher_visible = true
            "#,
        )
        .expect("manifest should deserialize");

        assert_eq!(
            manifest.scripting,
            Some(ModScriptingManifest {
                mod_script: Some("scripts/mod.rhai".to_owned()),
                mod_script_mode: ModScriptMode::Persistent,
            })
        );
        assert_eq!(manifest.scenes[0].path, "scenes/basic-scripting-demo");
        assert_eq!(manifest.scenes[0].document, None);
        assert_eq!(manifest.scenes[0].script, None);
    }

    #[test]
    fn resolves_scene_folder_paths_from_canonical_scene_root() {
        let discovered_mod = DiscoveredMod {
            manifest: ModManifest {
                id: "playground-2d".to_owned(),
                name: "Playground 2D".to_owned(),
                version: "0.1.0".to_owned(),
                description: None,
                authors: Vec::new(),
                dependencies: vec!["core".to_owned()],
                capabilities: vec!["rendering_2d".to_owned()],
                scripting: Some(ModScriptingManifest {
                    mod_script: Some("scripts/mod.rhai".to_owned()),
                    mod_script_mode: ModScriptMode::Persistent,
                }),
                scenes: vec![ModSceneManifest {
                    id: "basic-scripting-demo".to_owned(),
                    label: "Basic Scripting Demo".to_owned(),
                    description: None,
                    path: "scenes/basic-scripting-demo".to_owned(),
                    document: None,
                    script: None,
                    launcher_visible: true,
                }],
            },
            root_path: PathBuf::from("mods/playground-2d"),
        };

        assert_eq!(
            discovered_mod.mod_script_path(),
            Some(PathBuf::from("mods/playground-2d/scripts/mod.rhai"))
        );
        assert_eq!(
            discovered_mod.scene_root_path("basic-scripting-demo"),
            Some(PathBuf::from("mods/playground-2d/scenes/basic-scripting-demo"))
        );
        assert_eq!(
            discovered_mod.scene_document_path("basic-scripting-demo"),
            Some(PathBuf::from(
                "mods/playground-2d/scenes/basic-scripting-demo/scene.yml"
            ))
        );
        assert_eq!(
            discovered_mod.scene_script_path("basic-scripting-demo"),
            Some(PathBuf::from(
                "mods/playground-2d/scenes/basic-scripting-demo/scene.rhai"
            ))
        );
    }

    #[test]
    fn resolves_scene_level_document_and_script_overrides_relative_to_scene_root() {
        let scene = ModSceneManifest {
            id: "hello-world-square".to_owned(),
            label: "Hello World Square".to_owned(),
            description: None,
            path: "scenes/hello-world-square".to_owned(),
            document: Some("custom-scene.yml".to_owned()),
            script: Some("logic/scene.rhai".to_owned()),
            launcher_visible: true,
        };
        let mod_root = Path::new("mods/playground-2d");

        assert_eq!(
            scene.root_path(mod_root),
            PathBuf::from("mods/playground-2d/scenes/hello-world-square")
        );
        assert_eq!(
            scene.document_path(mod_root),
            PathBuf::from("mods/playground-2d/scenes/hello-world-square/custom-scene.yml")
        );
        assert_eq!(
            scene.script_path(mod_root),
            PathBuf::from("mods/playground-2d/scenes/hello-world-square/logic/scene.rhai")
        );
    }

    #[test]
    fn resolves_selected_mods_with_dependencies_first() {
        let discovered = BTreeMap::from([
            ("core".to_owned(), discovered_mod("core", &[])),
            (
                "playground-2d".to_owned(),
                discovered_mod("playground-2d", &["core"]),
            ),
            (
                "playground-3d".to_owned(),
                discovered_mod("playground-3d", &["core"]),
            ),
        ]);

        let resolved = resolve_discovered_mods(&discovered, &[String::from("playground-2d")])
            .expect("dependency resolution should succeed");
        let resolved_ids = resolved
            .iter()
            .map(|discovered_mod| discovered_mod.manifest.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(resolved_ids, vec!["core", "playground-2d"]);
    }

    #[test]
    fn rejects_missing_dependencies() {
        let discovered = BTreeMap::from([(
            "playground-2d".to_owned(),
            discovered_mod("playground-2d", &["core"]),
        )]);

        let error = resolve_discovered_mods(&discovered, &[String::from("playground-2d")])
            .expect_err("missing dependency should fail");

        assert_eq!(
            error.to_string(),
            "mod `playground-2d` depends on missing mod `core`"
        );
    }

    #[test]
    fn rejects_dependency_cycles() {
        let discovered = BTreeMap::from([
            (
                "core".to_owned(),
                discovered_mod("core", &["playground-2d"]),
            ),
            (
                "playground-2d".to_owned(),
                discovered_mod("playground-2d", &["core"]),
            ),
        ]);

        let error = resolve_discovered_mods(&discovered, &[String::from("core")])
            .expect_err("cyclic dependencies should fail");

        assert_eq!(
            error.to_string(),
            "dependency cycle detected while resolving mod `core`"
        );
    }

    #[test]
    fn explicit_empty_selection_loads_no_mods() {
        let discovered = BTreeMap::from([
            ("core".to_owned(), discovered_mod("core", &[])),
            (
                "playground-2d".to_owned(),
                discovered_mod("playground-2d", &["core"]),
            ),
        ]);

        let resolved = resolve_discovered_mods(&discovered, &[])
            .expect("empty explicit selection should succeed");

        assert!(resolved.is_empty());
    }

    #[test]
    fn root_mod_selection_always_includes_core() {
        assert_eq!(requested_mods_for_root("core"), vec!["core".to_owned()]);
        assert_eq!(
            requested_mods_for_root("playground-2d"),
            vec!["core".to_owned(), "playground-2d".to_owned()]
        );
    }
}
