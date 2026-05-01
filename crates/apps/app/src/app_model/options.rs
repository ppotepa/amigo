#[derive(Debug, Clone)]
pub struct BootstrapOptions {
    pub mods_root: PathBuf,
    pub active_mods: Option<Vec<String>>,
    pub startup_mod: Option<String>,
    pub startup_scene: Option<String>,
    pub dev_mode: bool,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        Self {
            mods_root: PathBuf::from("mods"),
            active_mods: None,
            startup_mod: None,
            startup_scene: None,
            dev_mode: false,
        }
    }
}

impl BootstrapOptions {
    pub fn new(mods_root: impl Into<PathBuf>) -> Self {
        Self {
            mods_root: mods_root.into(),
            ..Self::default()
        }
    }

    pub fn with_active_mods(mut self, active_mods: impl Into<Vec<String>>) -> Self {
        self.active_mods = Some(active_mods.into());
        self
    }

    pub fn with_startup_mod(mut self, startup_mod: impl Into<String>) -> Self {
        self.startup_mod = Some(startup_mod.into());
        self
    }

    pub fn with_startup_scene(mut self, startup_scene: impl Into<String>) -> Self {
        self.startup_scene = Some(startup_scene.into());
        self
    }

    pub fn with_dev_mode(mut self, dev_mode: bool) -> Self {
        self.dev_mode = dev_mode;
        self
    }
}

