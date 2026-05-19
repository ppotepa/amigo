use std::fs;
use std::path::{Path, PathBuf};

use amigo_plugin_api::PluginManifest;
use amigo_plugin_manifest::parse_plugin_manifest_str;

use crate::error::PluginLoadError;

pub fn load_plugin_manifests_from_plugins_dir(
    plugins_dir: &Path,
) -> Result<Vec<PluginManifest>, Vec<PluginLoadError>> {
    let mut manifests = Vec::new();
    let mut errors = Vec::new();

    let families = match fs::read_dir(plugins_dir) {
        Ok(entries) => entries,
        Err(error) => {
            return Err(vec![PluginLoadError::Io {
                path: plugins_dir.to_path_buf(),
                source: error,
            }]);
        }
    };

    for family in families.flatten() {
        let family_path = family.path();

        if !family_path.is_dir() {
            continue;
        }

        let plugins = match fs::read_dir(&family_path) {
            Ok(entries) => entries,
            Err(error) => {
                errors.push(PluginLoadError::Io {
                    path: family_path,
                    source: error,
                });
                continue;
            }
        };

        for plugin in plugins.flatten() {
            let plugin_path = plugin.path();

            if !plugin_path.is_dir() {
                continue;
            }

            let manifest_path = plugin_path.join("plugin.toml");

            if !manifest_path.exists() {
                continue;
            }

            match load_one(&manifest_path) {
                Ok(manifest) => manifests.push(manifest),
                Err(error) => errors.push(error),
            }
        }
    }

    if errors.is_empty() {
        Ok(manifests)
    } else {
        Err(errors)
    }
}

fn load_one(path: &Path) -> Result<PluginManifest, PluginLoadError> {
    let content = fs::read_to_string(path).map_err(|source| PluginLoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    parse_plugin_manifest_str(&content).map_err(|source| PluginLoadError::Parse {
        path: PathBuf::from(path),
        message: format!("{source:?}"),
    })
}
