use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use amigo_core::{AmigoError, AmigoResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CargoProfile {
    #[default]
    Dev,
    Release,
}

impl CargoProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Release => "release",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LauncherProfile {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cargo_profile: CargoProfile,
    #[serde(default = "default_window_backend")]
    pub window_backend: String,
    #[serde(default = "default_input_backend")]
    pub input_backend: String,
    #[serde(default = "default_render_backend")]
    pub render_backend: String,
    #[serde(default = "default_script_backend")]
    pub script_backend: String,
    #[serde(default, alias = "startup_mod")]
    pub root_mod: Option<String>,
    #[serde(default)]
    pub startup_scene: Option<String>,
    #[serde(default)]
    pub hosted_default: bool,
}

impl LauncherProfile {
    pub fn display_label(&self) -> &str {
        if self.label.is_empty() {
            &self.id
        } else {
            &self.label
        }
    }

    pub fn root_mod_or_core(&self) -> &str {
        self.root_mod.as_deref().unwrap_or("core")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LauncherConfig {
    #[serde(default = "default_active_profile")]
    pub active_profile: String,
    #[serde(default = "default_mods_root")]
    pub mods_root: String,
    #[serde(default = "default_profiles")]
    pub profiles: Vec<LauncherProfile>,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            active_profile: default_active_profile(),
            mods_root: default_mods_root(),
            profiles: default_profiles(),
        }
    }
}

impl LauncherConfig {
    pub fn load(path: impl AsRef<Path>) -> AmigoResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(path)?;

        let mut config = match toml::from_str::<Self>(&raw) {
            Ok(config) => config,
            Err(_) => {
                let legacy = toml::from_str::<LegacyLauncherConfig>(&raw)
                    .map_err(|error| AmigoError::Message(error.to_string()))?;
                legacy.into_current()
            }
        };

        config.normalize();
        Ok(config)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> AmigoResult<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw =
            toml::to_string_pretty(self).map_err(|error| AmigoError::Message(error.to_string()))?;
        fs::write(path, raw)?;
        Ok(())
    }

    pub fn validate_phase1(&self) -> AmigoResult<()> {
        if self.profiles.is_empty() {
            return Err(AmigoError::Message(
                "launcher config must define at least one profile".to_owned(),
            ));
        }

        let mut known_ids = BTreeSet::new();

        for profile in &self.profiles {
            if profile.id.trim().is_empty() {
                return Err(AmigoError::Message(
                    "launcher profile id must not be empty".to_owned(),
                ));
            }

            if !known_ids.insert(profile.id.clone()) {
                return Err(AmigoError::Message(format!(
                    "launcher profile `{}` is defined more than once",
                    profile.id
                )));
            }

            validate_backend("window_backend", &profile.window_backend, "winit")?;
            validate_backend("input_backend", &profile.input_backend, "winit")?;
            validate_backend("render_backend", &profile.render_backend, "wgpu")?;
            validate_backend("script_backend", &profile.script_backend, "rhai")?;
        }

        if self.profile_by_id(&self.active_profile).is_none() {
            return Err(AmigoError::Message(format!(
                "active launcher profile `{}` does not exist",
                self.active_profile
            )));
        }

        Ok(())
    }

    pub fn active_profile(&self) -> AmigoResult<&LauncherProfile> {
        self.profile_by_id(&self.active_profile).ok_or_else(|| {
            AmigoError::Message(format!(
                "active launcher profile `{}` does not exist",
                self.active_profile
            ))
        })
    }

    pub fn active_profile_mut(&mut self) -> AmigoResult<&mut LauncherProfile> {
        let active_profile = self.active_profile.clone();
        self.profile_mut_by_id(&active_profile).ok_or_else(|| {
            AmigoError::Message(format!(
                "active launcher profile `{}` does not exist",
                active_profile
            ))
        })
    }

    pub fn set_active_profile(&mut self, profile_id: &str) -> AmigoResult<()> {
        if self.profile_by_id(profile_id).is_none() {
            return Err(AmigoError::Message(format!(
                "launcher profile `{profile_id}` does not exist"
            )));
        }

        self.active_profile = profile_id.to_owned();
        Ok(())
    }

    pub fn profile_by_id(&self, profile_id: &str) -> Option<&LauncherProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.id == profile_id)
    }

    fn profile_mut_by_id(&mut self, profile_id: &str) -> Option<&mut LauncherProfile> {
        self.profiles
            .iter_mut()
            .find(|profile| profile.id == profile_id)
    }

    fn normalize(&mut self) {
        if self.profiles.is_empty() {
            self.profiles = default_profiles();
        }

        for profile in &mut self.profiles {
            normalize_profile(profile);
        }

        if self.active_profile.trim().is_empty() {
            self.active_profile = self
                .profiles
                .first()
                .map(|profile| profile.id.clone())
                .unwrap_or_else(default_active_profile);
        }

        if self.profile_by_id(&self.active_profile).is_none() {
            if let Some(first_profile) = self.profiles.first() {
                self.active_profile = first_profile.id.clone();
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyLauncherConfig {
    #[serde(default = "default_profile")]
    profile: String,
    #[serde(default = "default_window_backend")]
    window_backend: String,
    #[serde(default = "default_input_backend")]
    input_backend: String,
    #[serde(default = "default_render_backend")]
    render_backend: String,
    #[serde(default = "default_script_backend")]
    script_backend: String,
    #[serde(default = "default_mods")]
    mods: Vec<String>,
    #[serde(default)]
    startup_mod: Option<String>,
    #[serde(default)]
    startup_scene: Option<String>,
    #[serde(default = "default_mods_root")]
    mods_root: String,
    #[serde(default)]
    hosted_default: bool,
}

impl LegacyLauncherConfig {
    fn into_current(self) -> LauncherConfig {
        LauncherConfig {
            active_profile: self.profile.clone(),
            mods_root: self.mods_root,
            profiles: vec![LauncherProfile {
                id: self.profile,
                label: "Legacy Imported Profile".to_owned(),
                description: "Migrated from the previous single-profile launcher config."
                    .to_owned(),
                cargo_profile: CargoProfile::Dev,
                window_backend: self.window_backend,
                input_backend: self.input_backend,
                render_backend: self.render_backend,
                script_backend: self.script_backend,
                root_mod: self
                    .startup_mod
                    .or_else(|| derive_root_mod_from_legacy_mods(&self.mods)),
                startup_scene: self.startup_scene,
                hosted_default: self.hosted_default,
            }],
        }
    }
}

fn validate_backend(field: &str, actual: &str, expected: &str) -> AmigoResult<()> {
    if actual == expected {
        return Ok(());
    }

    Err(AmigoError::Message(format!(
        "Phase 1 launcher only supports `{expected}` for `{field}`, got `{actual}`"
    )))
}

fn default_active_profile() -> String {
    "dev".to_owned()
}

fn default_profile() -> String {
    "phase1-default".to_owned()
}

fn default_window_backend() -> String {
    "winit".to_owned()
}

fn default_input_backend() -> String {
    "winit".to_owned()
}

fn default_render_backend() -> String {
    "wgpu".to_owned()
}

fn default_script_backend() -> String {
    "rhai".to_owned()
}

fn default_mods_root() -> String {
    "mods".to_owned()
}

fn default_mods() -> Vec<String> {
    vec![
        "core".to_owned(),
        "core-game".to_owned(),
        "playground-2d".to_owned(),
        "playground-3d".to_owned(),
    ]
}

fn default_profiles() -> Vec<LauncherProfile> {
    vec![
        LauncherProfile {
            id: "dev".to_owned(),
            label: "Development".to_owned(),
            description: "Fast iteration profile with all playground mods enabled.".to_owned(),
            cargo_profile: CargoProfile::Dev,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("core-game".to_owned()),
            startup_scene: Some("dev-shell".to_owned()),
            hosted_default: false,
        },
        LauncherProfile {
            id: "dev-hosted".to_owned(),
            label: "Development Hosted".to_owned(),
            description: "Development profile that opens the first hosted wgpu window.".to_owned(),
            cargo_profile: CargoProfile::Dev,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("core-game".to_owned()),
            startup_scene: Some("dev-shell".to_owned()),
            hosted_default: true,
        },
        LauncherProfile {
            id: "release".to_owned(),
            label: "Release".to_owned(),
            description: "Release cargo profile with core content only.".to_owned(),
            cargo_profile: CargoProfile::Release,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("core".to_owned()),
            startup_scene: Some("bootstrap".to_owned()),
            hosted_default: false,
        },
        LauncherProfile {
            id: "release-hosted".to_owned(),
            label: "Release Hosted".to_owned(),
            description: "Release cargo profile with hosted window and core content.".to_owned(),
            cargo_profile: CargoProfile::Release,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("core".to_owned()),
            startup_scene: Some("bootstrap".to_owned()),
            hosted_default: true,
        },
        LauncherProfile {
            id: "wgpu-playground".to_owned(),
            label: "WGPU Playground".to_owned(),
            description: "Hosted playground preset for validating the canonical 3D scene."
                .to_owned(),
            cargo_profile: CargoProfile::Dev,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("playground-3d".to_owned()),
            startup_scene: Some("hello-world-cube".to_owned()),
            hosted_default: true,
        },
        LauncherProfile {
            id: "playground-2d".to_owned(),
            label: "Playground 2D".to_owned(),
            description: "Scene-centric 2D hosted playground with a scripting showcase scene."
                .to_owned(),
            cargo_profile: CargoProfile::Dev,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("playground-2d".to_owned()),
            startup_scene: Some("basic-scripting-demo".to_owned()),
            hosted_default: true,
        },
        LauncherProfile {
            id: "playground-3d".to_owned(),
            label: "Playground 3D".to_owned(),
            description: "Scene-centric 3D hosted playground.".to_owned(),
            cargo_profile: CargoProfile::Dev,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("playground-3d".to_owned()),
            startup_scene: Some("hello-world-cube".to_owned()),
            hosted_default: true,
        },
        LauncherProfile {
            id: "playground-sidescroller".to_owned(),
            label: "Playground Sidescroller".to_owned(),
            description:
                "Scene-centric 2D sidescroller vertical slice scaffold with tilemap and HUD."
                    .to_owned(),
            cargo_profile: CargoProfile::Dev,
            window_backend: default_window_backend(),
            input_backend: default_input_backend(),
            render_backend: default_render_backend(),
            script_backend: default_script_backend(),
            root_mod: Some("playground-sidescroller".to_owned()),
            startup_scene: Some("vertical-slice".to_owned()),
            hosted_default: true,
        },
    ]
}

fn normalize_profile(profile: &mut LauncherProfile) {
    if profile
        .root_mod
        .as_deref()
        .is_none_or(|root_mod| root_mod.trim().is_empty())
    {
        profile.root_mod = Some("core".to_owned());
    }

    if profile.label.trim().is_empty() {
        profile.label = profile.id.clone();
    }
}

fn derive_root_mod_from_legacy_mods(mods: &[String]) -> Option<String> {
    mods.iter()
        .find(|mod_id| mod_id.as_str() != "core")
        .cloned()
        .or_else(|| mods.first().cloned())
}

#[cfg(test)]
mod tests {
    use super::{CargoProfile, LauncherConfig};

    #[test]
    fn default_config_contains_multiple_profiles() {
        let config = LauncherConfig::default();

        assert_eq!(config.active_profile, "dev");
        assert!(config.profiles.len() >= 4);
        assert!(config.profile_by_id("release").is_some());
        assert!(config.profile_by_id("wgpu-playground").is_some());
        assert!(config.profile_by_id("playground-2d").is_some());
        assert!(config.profile_by_id("playground-3d").is_some());
        assert!(config.profile_by_id("playground-sidescroller").is_some());
    }

    #[test]
    fn active_profile_lookup_returns_selected_profile() {
        let mut config = LauncherConfig::default();
        config
            .set_active_profile("release")
            .expect("release profile should exist");

        let active = config
            .active_profile()
            .expect("active profile should exist");

        assert_eq!(active.id, "release");
        assert_eq!(active.cargo_profile, CargoProfile::Release);
        assert_eq!(active.root_mod.as_deref(), Some("core"));
    }
}
