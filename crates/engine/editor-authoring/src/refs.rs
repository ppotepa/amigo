use std::path::{Component, Path, PathBuf};

use amigo_core::{AmigoError, AmigoResult};
use serde_yaml::Value;

#[derive(Debug, Clone)]
pub struct UseRef {
    pub group: String,
    pub raw: String,
    pub path: PathBuf,
}

pub fn collect_use_refs(
    scene_file: &Path,
    mod_root: &Path,
    value: &Value,
) -> AmigoResult<Vec<UseRef>> {
    let Some(use_value) = mapping_get(value, "use").or_else(|| mapping_get(value, "uses")) else {
        return Ok(Vec::new());
    };

    let mut refs = Vec::new();
    collect_use_value(scene_file, mod_root, "use", use_value, &mut refs)?;
    Ok(refs)
}

fn collect_use_value(
    scene_file: &Path,
    mod_root: &Path,
    group: &str,
    value: &Value,
    out: &mut Vec<UseRef>,
) -> AmigoResult<()> {
    match value {
        Value::String(raw) => out.push(UseRef {
            group: group.to_owned(),
            raw: raw.clone(),
            path: resolve_reference(scene_file, mod_root, raw)?,
        }),
        Value::Sequence(items) => {
            for item in items {
                collect_use_value(scene_file, mod_root, group, item, out)?;
            }
        }
        Value::Mapping(mapping) => {
            for (key, value) in mapping {
                let group = key.as_str().unwrap_or(group);
                collect_use_value(scene_file, mod_root, group, value, out)?;
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn mapping_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}

pub fn resolve_reference(base_path: &Path, mod_root: &Path, value: &str) -> AmigoResult<PathBuf> {
    if let Some(rest) = value.strip_prefix("mod:") {
        reject_unsafe_relative(rest)?;
        return Ok(resolve_with_yaml_default_extension(&mod_root.join(rest)));
    }

    reject_unsafe_relative(value)?;
    let base = base_path.parent().unwrap_or_else(|| Path::new(""));
    Ok(resolve_with_yaml_default_extension(&base.join(value)))
}

fn reject_unsafe_relative(value: &str) -> AmigoResult<()> {
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(AmigoError::Message(format!(
            "unsafe absolute scene reference `{value}`"
        )));
    }

    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(AmigoError::Message(format!(
                "unsafe scene reference `{value}`"
            )));
        }
    }

    Ok(())
}

fn resolve_with_yaml_default_extension(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        return path.to_path_buf();
    }

    let yml = path.with_extension("yml");
    if yml.exists() {
        return yml;
    }

    path.with_extension("yaml")
}
