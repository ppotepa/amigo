use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub trait PresetProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn snapshot(&self) -> Result<serde_yaml::Value, String>;
    /// Validate fully before replacing live state; failure must leave state unchanged.
    fn apply(&self, value: serde_yaml::Value) -> Result<(), String>;
    fn reset(&self) -> Result<(), String>;
}
#[derive(Serialize, Deserialize)]
struct Preset {
    version: u32,
    domain: String,
    values: serde_yaml::Value,
}
#[derive(Default)]
pub struct PresetService {
    providers: Mutex<BTreeMap<String, Arc<dyn PresetProvider>>>,
    directory: Mutex<Option<PathBuf>>,
    error: Mutex<Option<String>>,
    operation: Mutex<()>,
}
impl PresetService {
    pub fn report_error(&self, error: String) {
        *self.error.lock().unwrap() = Some(error);
    }
    pub fn take_error(&self) -> Option<String> {
        self.error.lock().unwrap().take()
    }
    pub fn register(&self, provider: Arc<dyn PresetProvider>) {
        self.providers
            .lock()
            .unwrap()
            .insert(provider.id().into(), provider);
    }
    pub fn set_directory(&self, path: PathBuf) {
        *self.directory.lock().unwrap() = Some(path);
    }
    fn provider(&self, domain: &str) -> Result<Arc<dyn PresetProvider>, String> {
        self.providers
            .lock()
            .unwrap()
            .get(domain)
            .cloned()
            .ok_or_else(|| format!("unknown preset domain {domain}"))
    }
    fn path(&self, name: &str) -> Result<PathBuf, String> {
        if name.is_empty()
            || name.len() > 80
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("preset name must contain 1–80 letters, digits, - or _".into());
        }
        Ok(self
            .directory
            .lock()
            .unwrap()
            .clone()
            .ok_or("no active scene preset directory")?
            .join(format!("{name}.yml")))
    }
    pub fn list(&self) -> Vec<String> {
        let Some(dir) = self.directory.lock().unwrap().clone() else {
            return vec![];
        };
        let mut names = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let p = entry.path();
                (p.extension().and_then(|v| v.to_str()) == Some("yml"))
                    .then(|| p.file_stem().unwrap().to_string_lossy().into_owned())
            })
            .collect::<Vec<_>>();
        names.sort();
        names
    }
    pub fn save(&self, domain: &str, name: &str, overwrite: bool) -> Result<(), String> {
        let _operation = self.operation.lock().unwrap();
        let path = self.path(name)?;
        let provider = self.provider(domain)?;
        let bytes = serde_yaml::to_string(&Preset {
            version: 1,
            domain: domain.into(),
            values: provider.snapshot()?,
        })
        .map_err(|e| e.to_string())?;
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        if path.exists() && !overwrite {
            return Err("preset exists; use explicit overwrite".into());
        }
        if path.exists() && overwrite {
            let existing: Preset =
                serde_yaml::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if existing.domain != domain {
                return Err("name belongs to another preset kind; choose a different name".into());
            }
        }
        let temp = path.with_extension(format!("{}.tmp", std::process::id()));
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&temp).map_err(|e| e.to_string())?;
            file.write_all(bytes.as_bytes())
                .and_then(|_| file.sync_all())
                .map_err(|e| e.to_string())?;
        }
        std::fs::rename(&temp, &path).map_err(|e| e.to_string())
    }
    pub fn list_for(&self, domain: &str) -> Vec<String> {
        self.list()
            .into_iter()
            .filter(|name| {
                self.path(name)
                    .ok()
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .and_then(|text| serde_yaml::from_str::<Preset>(&text).ok())
                    .is_some_and(|p| p.version == 1 && p.domain == domain)
            })
            .collect()
    }
    pub fn load(&self, domain: &str, name: &str) -> Result<(), String> {
        let path = self.path(name)?;
        let document: Preset =
            serde_yaml::from_str(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if document.version != 1 || document.domain != domain {
            return Err("preset version/domain mismatch".into());
        }
        self.provider(domain)?.apply(document.values)
    }
    pub fn reset(&self, domain: &str) -> Result<(), String> {
        self.provider(domain)?.reset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Example(Mutex<bool>);
    impl PresetProvider for Example {
        fn id(&self) -> &'static str {
            "example"
        }
        fn snapshot(&self) -> Result<serde_yaml::Value, String> {
            Ok(serde_yaml::Value::Bool(*self.0.lock().unwrap()))
        }
        fn apply(&self, v: serde_yaml::Value) -> Result<(), String> {
            let next = v.as_bool().ok_or("expected bool")?;
            *self.0.lock().unwrap() = next;
            Ok(())
        }
        fn reset(&self) -> Result<(), String> {
            *self.0.lock().unwrap() = false;
            Ok(())
        }
    }
    #[test]
    fn roundtrip_overwrite_path_validation_and_atomic_failure() {
        let root = std::env::temp_dir().join(format!("amigo-preset-test-{}", std::process::id()));
        let service = PresetService::default();
        service.set_directory(root.clone());
        let provider = Arc::new(Example(Mutex::new(true)));
        service.register(provider.clone());
        assert!(service.save("example", "../escape", false).is_err());
        service.save("example", "one", true).unwrap();
        assert!(service.save("example", "one", false).is_err());
        service.reset("example").unwrap();
        service.load("example", "one").unwrap();
        assert!(*provider.0.lock().unwrap());
        std::fs::write(
            root.join("bad.yml"),
            "version: 1\ndomain: example\nvalues: invalid",
        )
        .unwrap();
        assert!(service.load("example", "bad").is_err());
        assert!(*provider.0.lock().unwrap());
        assert_eq!(service.list(), vec!["bad", "one"]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
