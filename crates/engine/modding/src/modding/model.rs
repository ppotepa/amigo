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

