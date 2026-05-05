use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::asset_registry::dto::{
    AssetDomainDto, AssetRegistryDto, AssetRoleDto, AssetStatusDto, ManagedAssetDto,
};
use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};

pub fn build_asset_graph(mut registry: AssetRegistryDto) -> AssetRegistryDto {
    let mod_id = registry.mod_id.clone();
    let asset_keys = registry
        .managed_assets
        .iter()
        .map(|asset| asset.asset_key.clone())
        .collect::<BTreeSet<_>>();
    let mut used_by = BTreeMap::<String, BTreeSet<String>>::new();
    let mut registry_diagnostics = Vec::new();

    for asset in &mut registry.managed_assets {
        asset.domain = domain_for_asset(asset);
        asset.parent_key = parent_key_for_asset(&mod_id, asset);
        asset.role = role_for_asset(asset);

        if let Some(parent_key) = &asset.parent_key {
            if !asset_keys.contains(parent_key) {
                let diagnostic = diagnostic(
                    DiagnosticLevel::Warning,
                    "asset_parent_missing",
                    format!(
                        "Asset `{}` references missing parent `{parent_key}`.",
                        asset.asset_key
                    ),
                    Some(asset.descriptor_relative_path.clone()),
                );
                asset.diagnostics.push(diagnostic.clone());
                registry_diagnostics.push(diagnostic);
                if matches!(asset.status, AssetStatusDto::Valid) {
                    asset.status = AssetStatusDto::Warning;
                }
            }
        }

        let mut references = asset
            .source_files
            .iter()
            .map(|source| source.relative_path.clone())
            .collect::<BTreeSet<_>>();

        for (field, value) in descriptor_references(asset) {
            if let Some(reference_key) =
                resolve_reference_key(&mod_id, asset, &field, &value, &asset_keys)
            {
                references.insert(reference_key);
            }
        }

        asset.references = references.into_iter().collect();
        for reference in &asset.references {
            if !asset_keys.contains(reference) && !reference.starts_with("raw/") {
                let diagnostic = diagnostic(
                    DiagnosticLevel::Warning,
                    "asset_reference_missing",
                    format!(
                        "Asset `{}` references missing asset `{reference}`.",
                        asset.asset_key
                    ),
                    Some(asset.descriptor_relative_path.clone()),
                );
                asset.diagnostics.push(diagnostic.clone());
                registry_diagnostics.push(diagnostic);
                if matches!(asset.status, AssetStatusDto::Valid) {
                    asset.status = AssetStatusDto::Warning;
                }
            }
        }
        for reference in &asset.references {
            if asset_keys.contains(reference) {
                used_by
                    .entry(reference.clone())
                    .or_default()
                    .insert(asset.asset_key.clone());
            }
        }
    }

    for asset in &mut registry.managed_assets {
        asset.used_by = used_by
            .remove(&asset.asset_key)
            .unwrap_or_default()
            .into_iter()
            .collect();
    }

    registry.diagnostics.extend(registry_diagnostics);
    registry.managed_assets.sort_by(|left, right| {
        domain_order(left.domain)
            .cmp(&domain_order(right.domain))
            .then_with(|| role_order(left.role).cmp(&role_order(right.role)))
            .then_with(|| left.asset_key.cmp(&right.asset_key))
    });
    registry
}

fn domain_for_asset(asset: &ManagedAssetDto) -> AssetDomainDto {
    match asset.kind.as_str() {
        "spritesheet-2d" | "tileset-2d" | "tile-ruleset-2d" | "animation-set-2d" => {
            AssetDomainDto::Spritesheet
        }
        "tilemap-2d" => AssetDomainDto::Tilemap,
        "audio" => AssetDomainDto::Audio,
        "font-2d" => AssetDomainDto::Font,
        "scene" => AssetDomainDto::Scene,
        "script" => AssetDomainDto::Script,
        _ => AssetDomainDto::Raw,
    }
}

fn role_for_asset(asset: &ManagedAssetDto) -> AssetRoleDto {
    if asset.parent_key.is_some() {
        return AssetRoleDto::Subasset;
    }
    match asset.domain {
        AssetDomainDto::Spritesheet
            if matches!(
                asset.kind.as_str(),
                "tileset-2d" | "tile-ruleset-2d" | "animation-set-2d"
            ) =>
        {
            AssetRoleDto::Subasset
        }
        AssetDomainDto::Script | AssetDomainDto::Raw => AssetRoleDto::File,
        _ => AssetRoleDto::Family,
    }
}

fn parent_key_for_asset(mod_id: &str, asset: &ManagedAssetDto) -> Option<String> {
    let normalized = asset.descriptor_relative_path.replace('\\', "/");
    let parts = normalized.split('/').collect::<Vec<_>>();
    if parts.len() >= 3 && parts.first().copied() == Some("scenes") && normalized.ends_with(".rhai") {
        return Some(format!("{mod_id}/scenes/{}", parts[1]));
    }
    if parts.len() < 4 || parts.first().copied() != Some("spritesheets") {
        return None;
    }
    let family_id = parts.get(1)?;
    if parts
        .iter()
        .any(|part| matches!(*part, "tilesets" | "rulesets" | "animations"))
    {
        return Some(format!("{mod_id}/spritesheets/{family_id}"));
    }
    None
}

fn descriptor_references(asset: &ManagedAssetDto) -> Vec<(String, String)> {
    let Ok(source) = std::fs::read_to_string(Path::new(&asset.descriptor_path)) else {
        return Vec::new();
    };
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&source) else {
        return Vec::new();
    };
    let json = yaml_to_json(yaml);
    let mut references = Vec::new();
    match asset.kind.as_str() {
        "tilemap-2d" => collect_named_reference_values(&json, &["tileset", "ruleset"], &mut references),
        "tile-ruleset-2d" => collect_named_reference_values(&json, &["tileset"], &mut references),
        "tileset-2d" => collect_named_reference_values(&json, &["spritesheet"], &mut references),
        "scene" => collect_scene_reference_values(&json, &mut references),
        "script" => collect_named_reference_values(&json, &["script", "scripts"], &mut references),
        _ => collect_named_reference_values(
            &json,
            &[
                "asset",
                "assets",
                "audio",
                "audios",
                "font",
                "fonts",
                "image",
                "images",
                "ruleset",
                "rulesets",
                "script",
                "scripts",
                "sprite",
                "sprites",
                "spritesheet",
                "spritesheets",
                "texture",
                "textures",
                "tileset",
                "tilesets",
            ],
            &mut references,
        ),
    }
    references
}

fn collect_named_reference_values(
    value: &Value,
    fields: &[&str],
    references: &mut Vec<(String, String)>,
) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if fields.iter().any(|field| reference_field_matches(key, field)) {
                    collect_reference_leaf_values(key, child, references);
                } else {
                    collect_named_reference_values(child, fields, references);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_named_reference_values(item, fields, references);
            }
        }
        _ => {}
    }
}

fn collect_scene_reference_values(value: &Value, references: &mut Vec<(String, String)>) {
    collect_named_reference_values(
        value,
        &[
            "asset",
            "assets",
            "audio",
            "audios",
            "font",
            "fonts",
            "image",
            "images",
            "ruleset",
            "rulesets",
            "script",
            "scripts",
            "sprite",
            "sprites",
            "spritesheet",
            "spritesheets",
            "texture",
            "textures",
            "tileset",
            "tilesets",
        ],
        references,
    );
}

fn collect_reference_leaf_values(
    field: &str,
    value: &Value,
    references: &mut Vec<(String, String)>,
) {
    match value {
        Value::String(value) if !value.trim().is_empty() => {
            references.push((field.to_owned(), value.to_owned()));
        }
        Value::Array(items) => {
            for item in items {
                collect_reference_leaf_values(field, item, references);
            }
        }
        Value::Object(object) => {
            for child in object.values() {
                collect_reference_leaf_values(field, child, references);
            }
        }
        _ => {}
    }
}

fn reference_field_matches(key: &str, field: &str) -> bool {
    key == field || key.strip_suffix(field).is_some_and(|prefix| prefix.ends_with('_'))
}

fn resolve_reference_key(
    mod_id: &str,
    asset: &ManagedAssetDto,
    field: &str,
    value: &str,
    asset_keys: &BTreeSet<String>,
) -> Option<String> {
    let normalized = normalize_reference(value);
    if normalized.is_empty() || normalized.starts_with("raw/") {
        return None;
    }

    let candidates = reference_key_candidates(mod_id, asset, field, &normalized);
    candidates
        .iter()
        .find(|candidate| asset_keys.contains(*candidate))
        .cloned()
        .or_else(|| candidates.into_iter().next())
}

fn reference_key_candidates(
    mod_id: &str,
    asset: &ManagedAssetDto,
    field: &str,
    value: &str,
) -> Vec<String> {
    let mut candidates = Vec::new();
    let field = singular_reference_field(field);
    let value_parts = value.split('/').collect::<Vec<_>>();

    if value.starts_with(&format!("{mod_id}/")) || looks_like_qualified_asset_key(value) {
        candidates.push(value.to_owned());
    }
    if starts_with_asset_area(value) {
        candidates.push(format!("{mod_id}/{value}"));
    }

    match field {
        "spritesheet" | "sprite" | "image" | "texture" => {
            candidates.push(format!("{mod_id}/spritesheets/{value}"));
        }
        "tileset" => {
            if value_parts.len() >= 3 && value_parts.first() == Some(&"spritesheets") {
                candidates.push(format!("{mod_id}/{value}"));
            } else if value_parts.len() >= 2 {
                let family = value_parts[0];
                let rest = value_parts[1..].join("/");
                candidates.push(format!("{mod_id}/spritesheets/{family}/tilesets/{rest}"));
            }
            if let Some(family) = spritesheet_family_id(asset) {
                candidates.push(format!("{mod_id}/spritesheets/{family}/tilesets/{value}"));
            }
        }
        "ruleset" => {
            if value_parts.len() >= 3 && value_parts.first() == Some(&"spritesheets") {
                candidates.push(format!("{mod_id}/{value}"));
            } else if value_parts.len() >= 2 {
                let family = value_parts[0];
                let rest = value_parts[1..].join("/");
                candidates.push(format!("{mod_id}/spritesheets/{family}/rulesets/{rest}"));
            }
            if let Some(family) = spritesheet_family_id(asset) {
                candidates.push(format!("{mod_id}/spritesheets/{family}/rulesets/{value}"));
            }
        }
        "font" => {
            candidates.push(format!("{mod_id}/fonts/{value}"));
        }
        "audio" => {
            candidates.push(format!("{mod_id}/audio/{value}"));
        }
        "script" => {
            if asset.descriptor_relative_path.starts_with("scenes/") {
                if let Some(scene_id) = asset.descriptor_relative_path.split('/').nth(1) {
                    candidates.push(format!(
                        "{mod_id}/scenes/{scene_id}/scripts/{}",
                        value.trim_end_matches(".rhai")
                    ));
                }
            }
            if let Some(stripped) = value.strip_prefix("scripts/") {
                candidates.push(format!("{mod_id}/scripts/{}", stripped.trim_end_matches(".rhai")));
            } else {
                candidates.push(format!("{mod_id}/scripts/{}", value.trim_end_matches(".rhai")));
            }
        }
        "asset" => {
            candidates.push(format!("{mod_id}/{value}"));
        }
        _ => {}
    }

    dedupe(candidates)
}

fn normalize_reference(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_end_matches(".yaml")
        .trim_end_matches(".yml")
        .trim_end_matches(".rhai")
        .trim_end_matches(".tilemap")
        .to_owned()
}

fn singular_reference_field(field: &str) -> &str {
    match field {
        "assets" => "asset",
        "audios" => "audio",
        "fonts" => "font",
        "images" => "image",
        "rulesets" => "ruleset",
        "scripts" => "script",
        "sprites" => "sprite",
        "spritesheets" => "spritesheet",
        "textures" => "texture",
        "tilesets" => "tileset",
        other => other,
    }
}

fn starts_with_asset_area(value: &str) -> bool {
    matches!(
        value.split('/').next(),
        Some("audio" | "data" | "fonts" | "packages" | "scenes" | "scripts" | "spritesheets")
    )
}

fn looks_like_qualified_asset_key(value: &str) -> bool {
    matches!(
        value.split('/').nth(1),
        Some("audio" | "data" | "fonts" | "packages" | "scenes" | "scripts" | "spritesheets")
    )
}

fn spritesheet_family_id(asset: &ManagedAssetDto) -> Option<String> {
    let normalized = asset.descriptor_relative_path.replace('\\', "/");
    let parts = normalized.split('/').collect::<Vec<_>>();
    if parts.first().copied() == Some("spritesheets") {
        return parts.get(1).map(|value| (*value).to_owned());
    }
    None
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| !value.is_empty() && seen.insert(value.clone()))
        .collect()
}

fn domain_order(domain: AssetDomainDto) -> usize {
    match domain {
        AssetDomainDto::Scene => 0,
        AssetDomainDto::Spritesheet => 1,
        AssetDomainDto::Tilemap => 2,
        AssetDomainDto::Audio => 3,
        AssetDomainDto::Font => 4,
        AssetDomainDto::Script => 5,
        AssetDomainDto::Raw => 6,
    }
}

fn role_order(role: AssetRoleDto) -> usize {
    match role {
        AssetRoleDto::Family => 0,
        AssetRoleDto::Subasset => 1,
        AssetRoleDto::Reference => 2,
        AssetRoleDto::File => 3,
    }
}

fn diagnostic(
    level: DiagnosticLevel,
    code: &str,
    message: impl Into<String>,
    path: Option<String>,
) -> EditorDiagnosticDto {
    EditorDiagnosticDto {
        level,
        code: code.to_owned(),
        message: message.into(),
        path,
    }
}

fn yaml_to_json(value: serde_yaml::Value) -> Value {
    match value {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(value) => Value::Bool(value),
        serde_yaml::Value::Number(value) => {
            if let Some(value) = value.as_u64() {
                Value::Number(value.into())
            } else if let Some(value) = value.as_i64() {
                Value::Number(value.into())
            } else if let Some(value) = value.as_f64() {
                serde_json::Number::from_f64(value)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        serde_yaml::Value::String(value) => Value::String(value),
        serde_yaml::Value::Sequence(items) => {
            Value::Array(items.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut object = serde_json::Map::new();
            for (key, value) in map {
                object.insert(yaml_key_to_string(key), yaml_to_json(value));
            }
            Value::Object(object)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}

fn yaml_key_to_string(value: serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(value) => value,
        serde_yaml::Value::Number(value) => value.to_string(),
        serde_yaml::Value::Bool(value) => value.to_string(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::build_asset_graph;
    use crate::asset_registry::dto::{AssetDomainDto, AssetRoleDto};
    use crate::asset_registry::scanner::scan_asset_registry;

    #[test]
    fn builds_spritesheet_tilemap_raw_and_orphan_relations() {
        let root = test_root("graph");
        write_file(root.join("raw/images/dirt.png"), "");
        write_file(root.join("raw/images/unused.png"), "");
        write_file(
            root.join("spritesheets/dirt/spritesheet.yml"),
            r#"
kind: spritesheet-2d
schema_version: 1
id: dirt
label: Dirt
source: raw/images/dirt.png
"#,
        );
        write_file(
            root.join("spritesheets/dirt/tilesets/platform/base.yml"),
            r#"
kind: tileset-2d
schema_version: 1
id: platform/base
label: Dirt Platform Base
spritesheet: dirt
"#,
        );
        write_file(
            root.join("spritesheets/dirt/rulesets/platform/solid.yml"),
            r#"
kind: tile-ruleset-2d
schema_version: 1
id: platform/solid
label: Dirt Platform Rules
tileset: dirt/platform/base
"#,
        );
        write_file(
            root.join("data/tilemaps/level-01.tilemap.yml"),
            r#"
kind: tilemap-2d
schema_version: 1
id: level-01
label: Level 01
tileset: graph-mod/spritesheets/dirt/tilesets/platform/base
ruleset: graph-mod/spritesheets/dirt/rulesets/platform/solid
"#,
        );
        write_file(
            root.join("scenes/level-01/scene.yml"),
            r#"
version: 1
scene:
  id: level-01
  label: Level 01 Scene
imports:
  scripts:
    - scene.rhai
entities:
  - id: terrain
    components:
      - type: Tilemap2D
        tileset: graph-mod/spritesheets/dirt/tilesets/platform/base
        ruleset: graph-mod/spritesheets/dirt/rulesets/platform/solid
      - type: Text2D
        font: graph-mod/fonts/missing
"#,
        );
        write_file(root.join("scenes/level-01/scene.rhai"), "");

        let registry =
            build_asset_graph(scan_asset_registry("session-1", "graph-mod", &root).unwrap());

        let spritesheet = asset(&registry, "graph-mod/spritesheets/dirt");
        assert_eq!(spritesheet.domain, AssetDomainDto::Spritesheet);
        assert_eq!(spritesheet.role, AssetRoleDto::Family);
        assert!(spritesheet.references.contains(&"raw/images/dirt.png".to_owned()));

        let tileset = asset(&registry, "graph-mod/spritesheets/dirt/tilesets/platform/base");
        assert_eq!(
            tileset.parent_key.as_deref(),
            Some("graph-mod/spritesheets/dirt")
        );

        let ruleset = asset(&registry, "graph-mod/spritesheets/dirt/rulesets/platform/solid");
        assert_eq!(
            ruleset.parent_key.as_deref(),
            Some("graph-mod/spritesheets/dirt")
        );
        assert!(ruleset
            .references
            .contains(&"graph-mod/spritesheets/dirt/tilesets/platform/base".to_owned()));

        let tilemap = asset(&registry, "graph-mod/data/tilemaps/level-01");
        assert_eq!(tilemap.domain, AssetDomainDto::Tilemap);
        assert!(tilemap
            .references
            .contains(&"graph-mod/spritesheets/dirt/tilesets/platform/base".to_owned()));
        assert!(tileset
            .used_by
            .contains(&"graph-mod/spritesheets/dirt/rulesets/platform/solid".to_owned()));
        assert!(tileset
            .used_by
            .contains(&"graph-mod/data/tilemaps/level-01".to_owned()));

        let scene = asset(&registry, "graph-mod/scenes/level-01");
        assert_eq!(scene.domain, AssetDomainDto::Scene);
        assert!(scene
            .references
            .contains(&"graph-mod/scenes/level-01/scripts/scene".to_owned()));
        assert!(scene
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "asset_reference_missing"));

        let scene_script = asset(&registry, "graph-mod/scenes/level-01/scripts/scene");
        assert_eq!(scene_script.domain, AssetDomainDto::Script);
        assert_eq!(
            scene_script.parent_key.as_deref(),
            Some("graph-mod/scenes/level-01")
        );

        let dirt_raw = registry
            .raw_files
            .iter()
            .find(|file| file.relative_path == "raw/images/dirt.png")
            .unwrap();
        assert!(dirt_raw
            .referenced_by
            .contains(&"graph-mod/spritesheets/dirt".to_owned()));
        assert!(registry
            .raw_files
            .iter()
            .any(|file| file.relative_path == "raw/images/unused.png" && file.orphan));
        assert!(registry
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "raw_file_orphan"));
    }

    fn asset<'a>(
        registry: &'a crate::asset_registry::dto::AssetRegistryDto,
        asset_key: &str,
    ) -> &'a crate::asset_registry::dto::ManagedAssetDto {
        registry
            .managed_assets
            .iter()
            .find(|asset| asset.asset_key == asset_key)
            .unwrap_or_else(|| panic!("asset `{asset_key}` was not found"))
    }

    fn test_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("amigo-editor-asset-graph-test-{name}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_file(path: PathBuf, content: &str) {
        ensure_parent(&path);
        fs::write(path, content).unwrap();
    }

    fn ensure_parent(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
    }
}
