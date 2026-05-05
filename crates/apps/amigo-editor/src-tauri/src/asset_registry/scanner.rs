use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::asset_registry::dto::{
    AssetDomainDto, AssetRegistryDto, AssetRoleDto, AssetSourceRefDto, AssetStatusDto,
    CreateAssetDescriptorRequestDto, CreateAssetImportOptionsDto,
    CreateSpritesheetRulesetRequestDto, ManagedAssetDto, RawAssetFileDto,
};
use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};

#[derive(Debug, Clone)]
struct DescriptorInfo {
    area: String,
    suffix: &'static str,
    expected_kind: &'static str,
}

const MVP_CREATABLE_DESCRIPTOR_KINDS: &[&str] = &["image", "tileset", "sprite"];
pub fn scan_asset_registry(
    session_id: &str,
    mod_id: &str,
    root: &Path,
) -> Result<AssetRegistryDto, String> {
    let mut managed_assets = Vec::new();
    let mut diagnostics = Vec::new();
    let mut source_refs_by_relative_path = BTreeMap::<String, Vec<String>>::new();

    for path in collect_files(root)? {
        if descriptor_info_for_path(&relative_path(root, &path)).is_none() {
            continue;
        }
        match read_descriptor(root, mod_id, &path) {
            Ok(asset) => {
                for source in &asset.source_files {
                    if source.exists {
                        source_refs_by_relative_path
                            .entry(source.relative_path.clone())
                            .or_default()
                            .push(asset.asset_key.clone());
                    }
                }
                diagnostics.extend(asset.diagnostics.clone());
                managed_assets.push(asset);
            }
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    let raw_candidates = collect_files(&root.join("raw"))?;
    let raw_files = raw_candidates
        .into_iter()
        .filter(|path| path.is_file())
        .map(|path| {
            let relative_path = relative_path(root, &path);
            let referenced_by = source_refs_by_relative_path
                .remove(&relative_path)
                .unwrap_or_default();
            RawAssetFileDto {
                media_type: media_type_for_path(&path),
                path: display_path(&path),
                relative_path,
                width: image::image_dimensions(&path).ok().map(|(width, _)| width),
                height: image::image_dimensions(&path).ok().map(|(_, height)| height),
                orphan: referenced_by.is_empty(),
                referenced_by,
            }
        })
        .collect::<Vec<_>>();

    for raw in &raw_files {
        if raw.orphan {
            diagnostics.push(diagnostic(
                DiagnosticLevel::Warning,
                "raw_file_orphan",
                format!(
                    "Raw file `{}` is not referenced by any descriptor.",
                    raw.relative_path
                ),
                Some(raw.relative_path.clone()),
            ));
        }
    }

    managed_assets.sort_by(|a, b| a.asset_key.cmp(&b.asset_key));
    let mut raw_files = raw_files;
    raw_files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(AssetRegistryDto {
        session_id: session_id.to_owned(),
        mod_id: mod_id.to_owned(),
        root_path: display_path(root),
        managed_assets,
        raw_files,
        diagnostics,
    })
}

pub fn create_asset_descriptor(
    mod_id: &str,
    root: &Path,
    request: CreateAssetDescriptorRequestDto,
) -> Result<ManagedAssetDto, String> {
    let raw_path = resolve_existing_project_file(root, &request.raw_file_path)?;
    let kind = normalize_descriptor_kind(&request.kind)?;
    let asset_id = sanitize_asset_id(&request.asset_id)?;
    let descriptor_path = descriptor_path_for_new_asset(root, &kind, &asset_id);
    if descriptor_path.exists() {
        return Err(format!(
            "asset descriptor `{}` already exists",
            descriptor_path.display()
        ));
    }
    if let Some(parent) = descriptor_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create asset descriptor parent `{}`: {error}",
                parent.display()
            )
        })?;
    }

    if kind == "tileset" {
        let spritesheet_path = root.join("spritesheets").join(&asset_id).join("spritesheet.yml");
        if !spritesheet_path.exists() {
            if let Some(parent) = spritesheet_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create spritesheet parent `{}`: {error}",
                        parent.display()
                    )
                })?;
            }
            let source_file = relative_between(spritesheet_path.parent().unwrap_or(root), &raw_path, root);
            let yaml = descriptor_yaml(
                "sprite",
                &asset_id,
                &source_file,
                &raw_path,
                request.import_options.as_ref(),
            )?;
            std::fs::write(&spritesheet_path, yaml).map_err(|error| {
                format!(
                    "failed to write parent spritesheet `{}`: {error}",
                    spritesheet_path.display()
                )
            })?;
        }
    }

    let source_file = relative_between(descriptor_path.parent().unwrap_or(root), &raw_path, root);
    let yaml = descriptor_yaml(
        &kind,
        &asset_id,
        &source_file,
        &raw_path,
        request.import_options.as_ref(),
    )?;
    std::fs::write(&descriptor_path, yaml).map_err(|error| {
        format!(
            "failed to write asset descriptor `{}`: {error}",
            descriptor_path.display()
        )
    })?;

    read_descriptor(root, mod_id, &descriptor_path).map_err(|diagnostic| diagnostic.message)
}

pub fn create_spritesheet_ruleset(
    mod_id: &str,
    root: &Path,
    request: CreateSpritesheetRulesetRequestDto,
) -> Result<ManagedAssetDto, String> {
    let family_id = spritesheet_family_from_asset_key(mod_id, &request.spritesheet_asset_key)?;
    let requested_id = request
        .ruleset_id
        .as_deref()
        .map(sanitize_ruleset_id)
        .transpose()?;
    let ruleset_id = requested_id.unwrap_or_else(|| next_ruleset_id(root, &family_id));
    let descriptor_path = root
        .join("spritesheets")
        .join(&family_id)
        .join("rulesets")
        .join(format!("{ruleset_id}.yml"));
    if descriptor_path.exists() {
        return Err(format!(
            "ruleset descriptor `{}` already exists",
            descriptor_path.display()
        ));
    }
    if let Some(parent) = descriptor_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create ruleset directory `{}`: {error}",
                parent.display()
            )
        })?;
    }

    let label = ruleset_id
        .split('/')
        .last()
        .map(title_label)
        .unwrap_or_else(|| "Ruleset".to_owned());
    let yaml = format!(
        "kind: tile-ruleset-2d\nschema_version: 1\nid: {ruleset_id}\nlabel: {label}\n\ntileset: platform/base\n\nterrains:\n  - id: solid\n    symbol: \"#\"\n    collision: solid\n    variants: {{}}\n"
    );
    std::fs::write(&descriptor_path, yaml).map_err(|error| {
        format!(
            "failed to write ruleset descriptor `{}`: {error}",
            descriptor_path.display()
        )
    })?;

    read_descriptor(root, mod_id, &descriptor_path).map_err(|diagnostic| diagnostic.message)
}

fn read_descriptor(
    root: &Path,
    mod_id: &str,
    path: &Path,
) -> Result<ManagedAssetDto, EditorDiagnosticDto> {
    let relative = relative_path(root, path);
    let Some(info) = descriptor_info_for_path(&relative) else {
        return Err(diagnostic(
            DiagnosticLevel::Error,
            "asset_descriptor_suffix_invalid",
            format!("Descriptor `{relative}` does not use a supported typed suffix."),
            Some(relative),
        ));
    };
    if info.expected_kind == "script" {
        return Ok(file_asset(root, mod_id, path, &relative, &info));
    }

    let source = std::fs::read_to_string(path).map_err(|error| {
        diagnostic(
            DiagnosticLevel::Error,
            "asset_descriptor_read_failed",
            format!("Failed to read `{}`: {error}", relative),
            Some(relative.clone()),
        )
    })?;
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(&source).map_err(|error| {
        diagnostic(
            DiagnosticLevel::Error,
            "asset_descriptor_parse_failed",
            format!("Failed to parse `{}`: {error}", relative),
            Some(relative.clone()),
        )
    })?;
    let value = yaml_to_json(yaml);
    let kind_value = string_at(&value, &["kind"]).unwrap_or_default();
    let mut diagnostics = Vec::new();
    if !kind_value.is_empty() && kind_value != info.expected_kind {
        diagnostics.push(diagnostic(
            DiagnosticLevel::Warning,
            "asset_descriptor_kind_mismatch",
            format!(
                "Descriptor kind `{kind_value}` should be `{}` for `*.{}.yml`.",
                info.expected_kind,
                info.suffix,
            ),
            Some(relative.clone()),
        ));
    }

    let asset_id = string_at(&value, &["id"])
        .or_else(|| string_at(&value, &["scene", "id"]))
        .unwrap_or_else(|| descriptor_stem_id(path, info.suffix));
    let label = string_at(&value, &["label"])
        .or_else(|| string_at(&value, &["scene", "label"]))
        .unwrap_or_else(|| asset_id.clone());
    let asset_key = descriptor_asset_key(mod_id, &relative, &info).unwrap_or_else(|| {
        format!("{mod_id}/{}/{}", info.area, asset_id)
    });
    let source_files = source_refs(root, path, &value);
    for source in &source_files {
        if !source.exists {
            diagnostics.push(diagnostic(
                DiagnosticLevel::Error,
                "asset_source_missing",
                format!("Asset source `{}` is missing.", source.relative_path),
                Some(relative.clone()),
            ));
        }
    }
    let status = if diagnostics
        .iter()
        .any(|item| matches!(item.level, DiagnosticLevel::Error))
    {
        if source_files.iter().any(|source| !source.exists) {
            AssetStatusDto::MissingSource
        } else {
            AssetStatusDto::Error
        }
    } else if diagnostics.is_empty() {
        AssetStatusDto::Valid
    } else {
        AssetStatusDto::Warning
    };

    Ok(ManagedAssetDto {
        asset_id,
        kind: info.expected_kind.to_owned(),
        label,
        asset_key,
        parent_key: None,
        references: Vec::new(),
        used_by: Vec::new(),
        domain: domain_for_kind(info.expected_kind),
        role: role_for_kind(info.expected_kind),
        descriptor_path: display_path(path),
        descriptor_relative_path: relative,
        source_files,
        status,
        diagnostics,
    })
}

fn file_asset(
    _root: &Path,
    mod_id: &str,
    path: &Path,
    relative: &str,
    info: &DescriptorInfo,
) -> ManagedAssetDto {
    let asset_id = script_asset_id(relative).unwrap_or_else(|| descriptor_stem_id(path, info.suffix));
    let label = asset_id
        .split('/')
        .last()
        .unwrap_or(asset_id.as_str())
        .to_owned();
    ManagedAssetDto {
        asset_id,
        kind: info.expected_kind.to_owned(),
        label,
        asset_key: descriptor_asset_key(mod_id, relative, info)
            .unwrap_or_else(|| format!("{mod_id}/{}/{}", info.area, descriptor_stem_id(path, info.suffix))),
        parent_key: None,
        references: Vec::new(),
        used_by: Vec::new(),
        domain: domain_for_kind(info.expected_kind),
        role: role_for_kind(info.expected_kind),
        descriptor_path: display_path(path),
        descriptor_relative_path: relative.to_owned(),
        source_files: Vec::new(),
        status: AssetStatusDto::Valid,
        diagnostics: Vec::new(),
    }
}

fn source_refs(root: &Path, descriptor_path: &Path, value: &Value) -> Vec<AssetSourceRefDto> {
    let mut paths = BTreeSet::new();
    for path in [
        value.get("source").and_then(Value::as_str).map(ToOwned::to_owned),
        string_at(value, &["source", "file"]),
        string_at(value, &["image"]),
        string_at(value, &["atlas", "image"]),
    ]
    .into_iter()
    .flatten()
    {
        if !path.trim().is_empty() {
            paths.insert(path);
        }
    }

    paths
        .into_iter()
        .map(|path| {
            let absolute = resolve_source_path(root, descriptor_path, &path);
            AssetSourceRefDto {
                exists: absolute.exists(),
                relative_path: relative_path(root, &absolute),
                path: display_path(&absolute),
                role: "source".to_owned(),
            }
        })
        .collect()
}

fn descriptor_info_for_path(relative_path: &str) -> Option<DescriptorInfo> {
    let normalized = relative_path.replace('\\', "/").to_ascii_lowercase();

    if normalized.starts_with("spritesheets/") {
        let parts = normalized.split('/').collect::<Vec<_>>();
        if parts.len() >= 3 && (parts[2] == "spritesheet.yml" || parts[2] == "spritesheet.yaml") {
            return Some(DescriptorInfo {
                area: "spritesheets".to_owned(),
                suffix: "spritesheet",
                expected_kind: "spritesheet-2d",
            });
        }
        if let Some(index) = parts.iter().position(|part| *part == "tilesets") {
            if normalized.ends_with(".yml") || normalized.ends_with(".yaml") {
                return Some(DescriptorInfo {
                    area: parts[..=index].join("/"),
                    suffix: "tileset",
                    expected_kind: "tileset-2d",
                });
            }
        }
        if let Some(index) = parts.iter().position(|part| *part == "rulesets") {
            if normalized.ends_with(".yml") || normalized.ends_with(".yaml") {
                return Some(DescriptorInfo {
                    area: parts[..=index].join("/"),
                    suffix: "tile-ruleset",
                    expected_kind: "tile-ruleset-2d",
                });
            }
        }
        if let Some(index) = parts.iter().position(|part| *part == "animations") {
            if normalized.ends_with(".yml") || normalized.ends_with(".yaml") {
                return Some(DescriptorInfo {
                    area: parts[..=index].join("/"),
                    suffix: "animation",
                    expected_kind: "animation-set-2d",
                });
            }
        }
    }

    if normalized.starts_with("fonts/") && normalized.ends_with("/font.yml") {
        return Some(DescriptorInfo {
            area: "fonts".to_owned(),
            suffix: "font",
            expected_kind: "font-2d",
        });
    }
    if normalized.starts_with("audio/") && normalized.ends_with("/audio.yml") {
        return Some(DescriptorInfo {
            area: "audio".to_owned(),
            suffix: "audio",
            expected_kind: "audio",
        });
    }
    if normalized.starts_with("data/tilemaps/")
        && (normalized.ends_with(".tilemap.yml") || normalized.ends_with(".tilemap.yaml"))
    {
        return Some(DescriptorInfo {
            area: "data/tilemaps".to_owned(),
            suffix: "tilemap",
            expected_kind: "tilemap-2d",
        });
    }
    if normalized.starts_with("scenes/") && normalized.ends_with("/scene.yml") {
        return Some(DescriptorInfo {
            area: "scenes".to_owned(),
            suffix: "scene",
            expected_kind: "scene",
        });
    }
    if normalized.starts_with("scenes/") && normalized.ends_with("/scene.yaml") {
        return Some(DescriptorInfo {
            area: "scenes".to_owned(),
            suffix: "scene",
            expected_kind: "scene",
        });
    }
    if normalized.starts_with("scenes/") && normalized.ends_with(".rhai") {
        return Some(DescriptorInfo {
            area: "scenes".to_owned(),
            suffix: "rhai",
            expected_kind: "script",
        });
    }
    if normalized.starts_with("scripts/") && normalized.ends_with(".rhai") {
        return Some(DescriptorInfo {
            area: "scripts".to_owned(),
            suffix: "rhai",
            expected_kind: "script",
        });
    }
    if normalized.starts_with("packages/") && (normalized.ends_with("/package.yml") || normalized.ends_with("/package.yaml")) {
        return Some(DescriptorInfo {
            area: "packages".to_owned(),
            suffix: "package",
            expected_kind: "script",
        });
    }

    None
}

fn domain_for_kind(kind: &str) -> AssetDomainDto {
    match kind {
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

fn role_for_kind(kind: &str) -> AssetRoleDto {
    match kind {
        "tileset-2d" | "tile-ruleset-2d" | "animation-set-2d" => AssetRoleDto::Subasset,
        "script" => AssetRoleDto::File,
        "spritesheet-2d" | "tilemap-2d" | "audio" | "font-2d" | "scene" => AssetRoleDto::Family,
        _ => AssetRoleDto::File,
    }
}

fn normalize_descriptor_kind(kind: &str) -> Result<String, String> {
    let normalized = kind
        .trim()
        .trim_start_matches('.')
        .trim_end_matches(".yml")
        .to_ascii_lowercase();
    let known = MVP_CREATABLE_DESCRIPTOR_KINDS
        .iter()
        .any(|suffix| *suffix == normalized);
    known
        .then_some(normalized)
        .ok_or_else(|| format!("asset descriptor kind `{kind}` is not supported"))
}

fn spritesheet_family_from_asset_key(mod_id: &str, asset_key: &str) -> Result<String, String> {
    let prefix = format!("{mod_id}/spritesheets/");
    let Some(remainder) = asset_key.strip_prefix(&prefix) else {
        return Err(format!(
            "asset `{asset_key}` is not a spritesheet family in mod `{mod_id}`"
        ));
    };
    let family = remainder
        .split('/')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("asset `{asset_key}` does not include a spritesheet id"))?;
    Ok(family.to_owned())
}

fn next_ruleset_id(root: &Path, family_id: &str) -> String {
    let directory = root
        .join("spritesheets")
        .join(family_id)
        .join("rulesets")
        .join("platform");
    if !directory.join("solid.yml").exists() {
        return "platform/solid".to_owned();
    }
    for index in 2..1000 {
        let id = format!("platform/solid-{index}");
        if !directory.join(format!("solid-{index}.yml")).exists() {
            return id;
        }
    }
    "platform/solid-extra".to_owned()
}

fn sanitize_ruleset_id(value: &str) -> Result<String, String> {
    let id = value
        .trim()
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .map(|part| sanitize_asset_id(part))
        .collect::<Result<Vec<_>, _>>()?
        .join("/");
    if id.is_empty() {
        return Err("ruleset id must not be empty".to_owned());
    }
    Ok(id)
}

fn descriptor_path_for_new_asset(root: &Path, kind: &str, asset_id: &str) -> PathBuf {
    match kind {
        "sprite" => root.join("spritesheets").join(asset_id).join("spritesheet.yml"),
        "image" => root.join("spritesheets").join(asset_id).join("spritesheet.yml"),
        "tileset" => root
            .join("spritesheets")
            .join(asset_id)
            .join("tilesets")
            .join("base.yml"),
        _ => root
            .join("custom")
            .join("assets")
            .join(format!("{asset_id}.{kind}.yml")),
    }
}

fn descriptor_asset_key(mod_id: &str, relative: &str, info: &DescriptorInfo) -> Option<String> {
    let normalized = relative.replace('\\', "/");
    if normalized.starts_with("spritesheets/") {
        if normalized.ends_with("/spritesheet.yml") || normalized.ends_with("/spritesheet.yaml") {
            let spritesheet_id = normalized.split('/').nth(1)?;
            return Some(format!("{mod_id}/spritesheets/{spritesheet_id}"));
        }
        if normalized.ends_with(".yml") || normalized.ends_with(".yaml") {
            return Some(format!(
                "{mod_id}/{}",
                normalized
                    .trim_end_matches(".yml")
                    .trim_end_matches(".yaml")
            ));
        }
    }
    if normalized.starts_with("fonts/") && normalized.ends_with("/font.yml") {
        let font_id = normalized.split('/').nth(1)?;
        return Some(format!("{mod_id}/fonts/{font_id}"));
    }
    if normalized.starts_with("audio/") && normalized.ends_with("/audio.yml") {
        let audio_id = normalized.split('/').nth(1)?;
        return Some(format!("{mod_id}/audio/{audio_id}"));
    }
    if normalized.starts_with("scenes/") && (normalized.ends_with("/scene.yml") || normalized.ends_with("/scene.yaml")) {
        let scene_id = normalized.split('/').nth(1)?;
        return Some(format!("{mod_id}/scenes/{scene_id}"));
    }
    if normalized.starts_with("scenes/") && normalized.ends_with(".rhai") {
        let scene_id = normalized.split('/').nth(1)?;
        let script_id = normalized
            .trim_start_matches(&format!("scenes/{scene_id}/"))
            .trim_end_matches(".rhai");
        return Some(format!("{mod_id}/scenes/{scene_id}/scripts/{script_id}"));
    }
    if normalized.starts_with("scripts/") && normalized.ends_with(".rhai") {
        return Some(format!(
            "{mod_id}/{}",
            normalized.trim_end_matches(".rhai")
        ));
    }
    if normalized.starts_with("packages/") && (normalized.ends_with("/package.yml") || normalized.ends_with("/package.yaml")) {
        let package_id = normalized.split('/').nth(1)?;
        return Some(format!("{mod_id}/packages/{package_id}"));
    }
    Some(format!("{mod_id}/{}/{}", info.area, descriptor_stem_id(Path::new(relative), info.suffix)))
}

fn script_asset_id(relative: &str) -> Option<String> {
    let normalized = relative.replace('\\', "/");
    if normalized.starts_with("scenes/") && normalized.ends_with(".rhai") {
        let parts = normalized.split('/').collect::<Vec<_>>();
        let scene_id = parts.get(1)?;
        let script_id = normalized
            .trim_start_matches(&format!("scenes/{scene_id}/"))
            .trim_end_matches(".rhai");
        return Some(format!("{scene_id}/scripts/{script_id}"));
    }
    if normalized.starts_with("scripts/") && normalized.ends_with(".rhai") {
        return Some(normalized.trim_start_matches("scripts/").trim_end_matches(".rhai").to_owned());
    }
    if normalized.starts_with("packages/") {
        return normalized.split('/').nth(1).map(ToOwned::to_owned);
    }
    None
}

fn descriptor_yaml(
    kind: &str,
    asset_id: &str,
    source_file: &str,
    raw_path: &Path,
    import_options: Option<&CreateAssetImportOptionsDto>,
) -> Result<String, String> {
    let label = title_label(asset_id);
    let source = source_file.replace('\\', "/");
    let image_size = image::image_dimensions(raw_path).ok();
    let source_block = if let Some((width, height)) = image_size {
        format!("  file: {source}\n  width: {width}\n  height: {height}")
    } else {
        format!("  file: {source}")
    };
    let yaml = match kind {
        "image" => format!(
            "kind: spritesheet-2d\nschema_version: 1\nid: {asset_id}\nlabel: {label}\n\nsource:\n{source_block}\n\ngrid:\n  tile_size: {{ x: {}, y: {} }}\n  columns: 1\n  rows: 1\n  frame_count: 1\n  margin: {{ x: 0, y: 0 }}\n  spacing: {{ x: 0, y: 0 }}\n\ndefaults:\n  pivot: center\n",
            image_size.map(|(width, _)| width).unwrap_or(1),
            image_size.map(|(_, height)| height).unwrap_or(1),
        ),
        "tileset" => {
            let (width, height) = image_size.unwrap_or((1, 1));
            let tile_width = import_options.and_then(|value| value.tile_width).unwrap_or(32);
            let tile_height = import_options.and_then(|value| value.tile_height).unwrap_or(32);
            let columns = import_options
                .and_then(|value| value.columns)
                .unwrap_or_else(|| std::cmp::max(1, width / std::cmp::max(1, tile_width)));
            let rows = import_options
                .and_then(|value| value.rows)
                .unwrap_or_else(|| std::cmp::max(1, height / std::cmp::max(1, tile_height)));
            let tile_count = import_options
                .and_then(|value| value.tile_count)
                .unwrap_or(columns.saturating_mul(rows));
            format!(
                "kind: tileset-2d\nschema_version: 1\nid: base\nlabel: {label} Base\n\nspritesheet: {asset_id}\n\nrange:\n  start: 0\n  count: {tile_count}\n\ntile_size: {{ x: {tile_width}, y: {tile_height} }}\n\ndefaults:\n  collision: solid\n  damageable: true\n\ntiles: {{}}\n"
            )
        }
        "sprite" => {
            let (width, height) = image_size.unwrap_or((1, 1));
            let tile_width = import_options.and_then(|value| value.tile_width).unwrap_or(32);
            let tile_height = import_options.and_then(|value| value.tile_height).unwrap_or(32);
            let columns = import_options
                .and_then(|value| value.columns)
                .unwrap_or_else(|| std::cmp::max(1, width / std::cmp::max(1, tile_width)));
            let rows = import_options
                .and_then(|value| value.rows)
                .unwrap_or_else(|| std::cmp::max(1, height / std::cmp::max(1, tile_height)));
            let frame_count = import_options
                .and_then(|value| value.tile_count)
                .unwrap_or(columns.saturating_mul(rows));
            let margin_x = import_options.and_then(|value| value.margin_x).unwrap_or(0);
            let margin_y = import_options.and_then(|value| value.margin_y).unwrap_or(0);
            let spacing_x = import_options.and_then(|value| value.spacing_x).unwrap_or(0);
            let spacing_y = import_options.and_then(|value| value.spacing_y).unwrap_or(0);
            let fps = import_options.and_then(|value| value.fps).unwrap_or(12);
            format!(
                "kind: spritesheet-2d\nschema_version: 1\nid: {asset_id}\nlabel: {label}\n\nsource:\n{source_block}\n\ngrid:\n  tile_size: {{ x: {tile_width}, y: {tile_height} }}\n  columns: {columns}\n  rows: {rows}\n  frame_count: {frame_count}\n  margin: {{ x: {margin_x}, y: {margin_y} }}\n  spacing: {{ x: {spacing_x}, y: {spacing_y} }}\n  fps: {fps}\n  looping: true\n\nanimations: {{}}\n"
            )
        }
        other => format!(
            "kind: {other}\nschema_version: 1\nid: {asset_id}\nlabel: {label}\n\nsource:\n{source_block}\n"
        ),
    };
    Ok(yaml)
}

fn collect_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_files_into(path, &mut files)?;
    Ok(files)
}

fn collect_files_into(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let Ok(metadata) = std::fs::metadata(path) else {
        return Ok(());
    };
    if metadata.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(path).map_err(|error| {
        format!(
            "failed to read asset registry directory `{}`: {error}",
            path.display()
        )
    })? {
        let entry = entry.map_err(|error| error.to_string())?;
        collect_files_into(&entry.path(), files)?;
    }
    Ok(())
}

fn resolve_existing_project_file(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let normalized = relative_path.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.starts_with('/') || normalized.contains("../") {
        return Err(format!(
            "asset path `{relative_path}` is not a safe project-relative path"
        ));
    }
    let candidate = root.join(normalized);
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            root.display()
        )
    })?;
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize asset path `{}`: {error}",
            candidate.display()
        )
    })?;
    if !canonical_candidate.starts_with(canonical_root) {
        return Err(format!("asset path `{relative_path}` escapes mod root"));
    }
    Ok(canonical_candidate)
}

fn resolve_source_path(root: &Path, descriptor_path: &Path, source: &str) -> PathBuf {
    let normalized = source.trim().replace('\\', "/");
    if normalized.starts_with("raw/") {
        return root.join(normalized);
    }
    descriptor_path.parent().unwrap_or(root).join(normalized)
}

fn relative_between(from_dir: &Path, to: &Path, root: &Path) -> String {
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let from_parts = from_dir
        .strip_prefix(&canonical_root)
        .or_else(|_| from_dir.strip_prefix(root))
        .unwrap_or(from_dir)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let to_parts = to
        .strip_prefix(&canonical_root)
        .or_else(|_| to.strip_prefix(root))
        .unwrap_or(to)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let common_len = from_parts
        .iter()
        .zip(&to_parts)
        .take_while(|(left, right)| left == right)
        .count();
    let mut parts = Vec::new();
    for _ in common_len..from_parts.len() {
        parts.push("..".to_owned());
    }
    parts.extend(to_parts.into_iter().skip(common_len));
    parts.join("/")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn display_path(path: &Path) -> String {
    let display_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = display_path.display().to_string();
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned()
}

fn media_type_for_path(path: &Path) -> String {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
    .to_owned()
}

fn descriptor_stem_id(path: &Path, suffix: &str) -> String {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("asset");
    file_name
        .trim_end_matches(".yml")
        .trim_end_matches(".yaml")
        .trim_end_matches(&format!(".{suffix}"))
        .to_owned()
}

fn sanitize_asset_id(value: &str) -> Result<String, String> {
    let id = value.trim().to_ascii_lowercase().replace('_', "-");
    let valid = !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
    valid
        .then_some(id)
        .ok_or_else(|| format!("asset id `{value}` must use lowercase letters, digits or `-`"))
}

fn title_label(id: &str) -> String {
    id.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))?
        .as_str()
        .map(ToOwned::to_owned)
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
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use image::{ImageBuffer, Rgba};

    use super::{create_asset_descriptor, create_spritesheet_ruleset, scan_asset_registry};
    use crate::asset_registry::dto::{
        AssetStatusDto, CreateAssetDescriptorRequestDto, CreateSpritesheetRulesetRequestDto,
    };

    #[test]
    fn scans_managed_raw_orphan_and_missing_sources() {
        let root = test_root("scan");
        write_png(root.join("raw/images/dirt.png"), 16, 16);
        write_png(root.join("raw/images/unused.png"), 16, 16);
        fs::write(
            root.join("spritesheets/dirt/spritesheet.yml"),
            r#"
kind: spritesheet-2d
schema_version: 1
id: dirt
label: Dirt
source: raw/images/dirt.png
grid:
  tile_size: { x: 16, y: 16 }
  columns: 1
  rows: 1
  frame_count: 1
"#,
        )
        .unwrap();
        fs::write(
            root.join("spritesheets/missing/spritesheet.yml"),
            r#"
kind: spritesheet-2d
schema_version: 1
id: missing
source: raw/images/missing.png
"#,
        )
        .unwrap();

        let registry = scan_asset_registry("session-1", "ink-wars", &root).unwrap();

        assert_eq!(registry.managed_assets.len(), 2);
        assert!(
            registry
                .raw_files
                .iter()
                .any(|file| file.relative_path == "raw/images/unused.png" && file.orphan)
        );
        assert!(registry.managed_assets.iter().any(|asset| {
            asset.asset_key == "ink-wars/spritesheets/missing"
                && matches!(asset.status, AssetStatusDto::MissingSource)
        }));
    }

    #[test]
    fn creates_descriptor_from_raw_file() {
        let root = test_root("create");
        write_png(root.join("raw/images/paper.png"), 32, 16);

        let asset = create_asset_descriptor(
            "ink-wars",
            &root,
            CreateAssetDescriptorRequestDto {
                raw_file_path: "raw/images/paper.png".to_owned(),
                kind: "sprite".to_owned(),
                asset_id: "paper".to_owned(),
                import_options: None,
            },
        )
        .unwrap();

        assert_eq!(asset.asset_key, "ink-wars/spritesheets/paper");
        assert_eq!(
            asset.descriptor_relative_path,
            "spritesheets/paper/spritesheet.yml"
        );
        let written = fs::read_to_string(root.join("spritesheets/paper/spritesheet.yml")).unwrap();
        assert!(written.contains("kind: spritesheet-2d"));
        assert!(written.contains("file: ../../raw/images/paper.png"));
    }

    #[test]
    fn creates_spritesheet_ruleset_descriptor() {
        let root = test_root("ruleset");
        fs::write(
            root.join("spritesheets/dirt/spritesheet.yml"),
            r#"
kind: spritesheet-2d
schema_version: 1
id: dirt
source: raw/images/dirt.png
"#,
        )
        .unwrap();

        let asset = create_spritesheet_ruleset(
            "ink-wars",
            &root,
            CreateSpritesheetRulesetRequestDto {
                spritesheet_asset_key: "ink-wars/spritesheets/dirt".to_owned(),
                ruleset_id: None,
            },
        )
        .unwrap();

        assert_eq!(
            asset.asset_key,
            "ink-wars/spritesheets/dirt/rulesets/platform/solid"
        );
        assert_eq!(
            asset.descriptor_relative_path,
            "spritesheets/dirt/rulesets/platform/solid.yml"
        );
        let written =
            fs::read_to_string(root.join("spritesheets/dirt/rulesets/platform/solid.yml"))
                .unwrap();
        assert!(written.contains("kind: tile-ruleset-2d"));
        assert!(written.contains("tileset: platform/base"));
    }

    fn test_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("amigo-editor-asset-registry-test-{name}-{stamp}"));
        fs::create_dir_all(root.join("raw/images")).unwrap();
        fs::create_dir_all(root.join("spritesheets/dirt")).unwrap();
        fs::create_dir_all(root.join("spritesheets/missing")).unwrap();
        root
    }

    fn write_png(path: PathBuf, width: u32, height: u32) {
        let image =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        image.save(path).unwrap();
    }
}
