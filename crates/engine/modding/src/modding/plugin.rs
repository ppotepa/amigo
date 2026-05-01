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

