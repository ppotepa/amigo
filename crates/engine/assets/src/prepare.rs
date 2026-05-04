use std::collections::BTreeMap;
use std::ffi::OsStr;

use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

use crate::{LoadedAsset, PreparedAsset, PreparedAssetKind};

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
    mut metadata: BTreeMap<String, String>,
) -> Result<PreparedAsset, String> {
    add_descriptor_first_aliases(&mut metadata);
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

fn add_descriptor_first_aliases(metadata: &mut BTreeMap<String, String>) {
    copy_metadata_value(metadata, "source.file", "image");
    copy_metadata_value(metadata, "source.width", "image_width");
    copy_metadata_value(metadata, "source.height", "image_height");
    copy_metadata_value(metadata, "atlas.image_size.width", "image_size.x");
    copy_metadata_value(metadata, "atlas.image_size.height", "image_size.y");
    copy_metadata_value(metadata, "atlas.tile_size.width", "tile_size.x");
    copy_metadata_value(metadata, "atlas.tile_size.height", "tile_size.y");
    copy_metadata_value(metadata, "atlas.frame_size.width", "frame_size.x");
    copy_metadata_value(metadata, "atlas.frame_size.height", "frame_size.y");
    copy_metadata_value(metadata, "atlas.tile_count", "tile_count");
    copy_metadata_value(metadata, "atlas.frame_count", "frame_count");
}

fn copy_metadata_value(metadata: &mut BTreeMap<String, String>, from: &str, to: &str) {
    if metadata.contains_key(to) {
        return;
    }
    if let Some(value) = metadata.get(from).cloned() {
        metadata.insert(to.to_owned(), value);
    }
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
    if value.len() >= 2 && value.starts_with('\"') && value.ends_with('\"') {
        return Ok(value[1..value.len() - 1].to_owned());
    }

    if value.contains('\"') {
        return Err(format!(
            "invalid quoted placeholder value in `{}`",
            loaded_asset.resolved_path.display()
        ));
    }

    Ok(value.to_owned())
}
