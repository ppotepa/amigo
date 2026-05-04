use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::asset_registry::dto::{
    AssetMigrationEntryDto, AssetMigrationPlanDto, AssetMigrationResultDto, AssetRegistryDto,
    AssetSourceRefDto, AssetStatusDto, CreateAssetDescriptorRequestDto,
    CreateAssetImportOptionsDto, ManagedAssetDto, RawAssetFileDto,
};
use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};

const DESCRIPTOR_AREAS: &[(&str, &str, &str)] = &[
    ("images", "image", "image-2d"),
    ("sprites", "sprite", "sprite-sheet-2d"),
    ("tilesets", "tileset", "tileset-2d"),
    ("tilesets", "tile-ruleset", "tile-ruleset-2d"),
    ("tilemaps", "tilemap", "tilemap-2d"),
    ("fonts", "font", "font-2d"),
    ("audio", "audio", "audio"),
    ("particles", "particle", "particle-2d"),
    ("materials", "material", "material"),
    ("ui", "ui", "ui"),
];

const MVP_CREATABLE_DESCRIPTOR_KINDS: &[&str] = &["image", "tileset", "sprite"];
const LEGACY_ROOTS: &[(&str, &str)] = &[
    ("textures", "image"),
    ("sprites", "sprite"),
    ("spritesheets", "sprite"),
    ("tilesets", "tileset"),
    ("tilemaps", "tilemap"),
];

pub fn scan_asset_registry(
    session_id: &str,
    mod_id: &str,
    root: &Path,
) -> Result<AssetRegistryDto, String> {
    let mut managed_assets = Vec::new();
    let mut diagnostics = Vec::new();
    let mut source_refs_by_relative_path = BTreeMap::<String, Vec<String>>::new();

    for path in collect_files(&root.join("assets"))? {
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

    let raw_files = collect_files(&root.join("assets").join("raw"))?
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
    let area = area_for_kind(&kind);
    let descriptor_path = root
        .join("assets")
        .join(area)
        .join(format!("{asset_id}.{kind}.yml"));
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

pub fn scan_asset_migration_plan(
    session_id: &str,
    mod_id: &str,
    root: &Path,
) -> Result<AssetMigrationPlanDto, String> {
    let mut entries = Vec::new();
    for (legacy_root, kind) in LEGACY_ROOTS {
        let directory = root.join(legacy_root);
        if !directory.exists() {
            continue;
        }
        for path in collect_files(&directory)? {
            if !path.is_file() {
                continue;
            }
            let relative = relative_path(root, &path);
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let file_stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("asset")
                .to_owned();
            if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                entries.push(AssetMigrationEntryDto {
                    action: "copy_raw".to_owned(),
                    from_path: Some(relative.clone()),
                    to_path: Some(format!("assets/raw/images/{}", path.file_name().and_then(|value| value.to_str()).unwrap_or_default())),
                    asset_kind: Some((*kind).to_owned()),
                    reason: format!("Move legacy raw asset out of `{legacy_root}/`."),
                });
                if *kind != "tilemap" {
                    let descriptor_name = format!("{file_stem}.{kind}.yml");
                    let area = area_for_kind(kind);
                    entries.push(AssetMigrationEntryDto {
                        action: "create_descriptor".to_owned(),
                        from_path: Some(relative),
                        to_path: Some(format!("assets/{area}/{descriptor_name}")),
                        asset_kind: Some((*kind).to_owned()),
                        reason: "Create descriptor-first asset.".to_owned(),
                    });
                }
            }
        }
    }
    Ok(AssetMigrationPlanDto {
        session_id: session_id.to_owned(),
        mod_id: mod_id.to_owned(),
        root_path: display_path(root),
        entries,
    })
}

pub fn apply_asset_migration_plan(
    root: &Path,
    plan: AssetMigrationPlanDto,
    dry_run: bool,
) -> Result<AssetMigrationResultDto, String> {
    let mut applied_entries = 0usize;
    for entry in &plan.entries {
        if dry_run {
            applied_entries += 1;
            continue;
        }
        match entry.action.as_str() {
            "copy_raw" => {
                let from = root.join(entry.from_path.as_deref().unwrap_or_default());
                let to = root.join(entry.to_path.as_deref().unwrap_or_default());
                if let Some(parent) = to.parent() {
                    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                if !to.exists() {
                    std::fs::copy(&from, &to).map_err(|error| error.to_string())?;
                }
            }
            "create_descriptor" => {
                let raw_source = entry.from_path.clone().unwrap_or_default();
                let kind = entry.asset_kind.clone().unwrap_or_else(|| "image".to_owned());
                let asset_id = Path::new(entry.to_path.as_deref().unwrap_or_default())
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("asset")
                    .split('.')
                    .next()
                    .unwrap_or("asset")
                    .to_owned();
                create_asset_descriptor(
                    &plan.mod_id,
                    root,
                    CreateAssetDescriptorRequestDto {
                        raw_file_path: raw_source,
                        kind,
                        asset_id,
                        import_options: None,
                    },
                )?;
            }
            _ => {}
        }
        applied_entries += 1;
    }

    let report_path = root.join(".amigo-editor").join("asset-migration-report.json");
    if !dry_run {
        if let Some(parent) = report_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let report = serde_json::to_string_pretty(&plan).map_err(|error| error.to_string())?;
        std::fs::write(&report_path, report).map_err(|error| error.to_string())?;
    }
    Ok(AssetMigrationResultDto {
        dry_run,
        applied_entries,
        report_path: (!dry_run).then_some(display_path(&report_path)),
    })
}

fn read_descriptor(
    root: &Path,
    mod_id: &str,
    path: &Path,
) -> Result<ManagedAssetDto, EditorDiagnosticDto> {
    let source = std::fs::read_to_string(path).map_err(|error| {
        diagnostic(
            DiagnosticLevel::Error,
            "asset_descriptor_read_failed",
            format!("Failed to read `{}`: {error}", relative_path(root, path)),
            Some(relative_path(root, path)),
        )
    })?;
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(&source).map_err(|error| {
        diagnostic(
            DiagnosticLevel::Error,
            "asset_descriptor_parse_failed",
            format!("Failed to parse `{}`: {error}", relative_path(root, path)),
            Some(relative_path(root, path)),
        )
    })?;
    let value = yaml_to_json(yaml);
    let kind_value = string_at(&value, &["kind"]).unwrap_or_default();
    let relative = relative_path(root, path);
    let Some((area, suffix, expected_kind)) = descriptor_info_for_path(&relative) else {
        return Err(diagnostic(
            DiagnosticLevel::Error,
            "asset_descriptor_suffix_invalid",
            format!("Descriptor `{relative}` does not use a supported typed suffix."),
            Some(relative),
        ));
    };
    let mut diagnostics = Vec::new();
    if kind_value != expected_kind {
        diagnostics.push(diagnostic(
            DiagnosticLevel::Warning,
            "asset_descriptor_kind_mismatch",
            format!(
                "Descriptor kind `{kind_value}` should be `{expected_kind}` for `*.{suffix}.yml`."
            ),
            Some(relative.clone()),
        ));
    }

    let asset_id = string_at(&value, &["id"]).unwrap_or_else(|| descriptor_stem_id(path, suffix));
    let label = string_at(&value, &["label"]).unwrap_or_else(|| asset_id.clone());
    let asset_key = format!("{mod_id}/{area}/{asset_id}");
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
        kind: expected_kind.to_owned(),
        label,
        asset_key,
        descriptor_path: display_path(path),
        descriptor_relative_path: relative,
        source_files,
        status,
        diagnostics,
    })
}

fn source_refs(root: &Path, descriptor_path: &Path, value: &Value) -> Vec<AssetSourceRefDto> {
    let mut paths = BTreeSet::new();
    for path in [
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

fn descriptor_info_for_path(
    relative_path: &str,
) -> Option<(&'static str, &'static str, &'static str)> {
    let normalized = relative_path.replace('\\', "/").to_ascii_lowercase();
    for (area, suffix, kind) in DESCRIPTOR_AREAS {
        if normalized.starts_with(&format!("assets/{area}/"))
            && (normalized.ends_with(&format!(".{suffix}.yml"))
                || normalized.ends_with(&format!(".{suffix}.yaml")))
        {
            return Some((*area, *suffix, *kind));
        }
    }
    None
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

fn area_for_kind(kind: &str) -> &'static str {
    DESCRIPTOR_AREAS
        .iter()
        .find_map(|(area, suffix, _)| (*suffix == kind).then_some(*area))
        .unwrap_or("other")
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
            "kind: image-2d\nschema_version: 1\nid: {asset_id}\nlabel: {label}\n\nsource:\n{source_block}\n\nusage: texture\n"
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
            let margin_x = import_options.and_then(|value| value.margin_x).unwrap_or(0);
            let margin_y = import_options.and_then(|value| value.margin_y).unwrap_or(0);
            let spacing_x = import_options.and_then(|value| value.spacing_x).unwrap_or(0);
            let spacing_y = import_options.and_then(|value| value.spacing_y).unwrap_or(0);
            format!(
                "kind: tileset-2d\nschema_version: 1\nid: {asset_id}\nlabel: {label}\n\nsource:\n{source_block}\n\natlas:\n  image_size: {{ width: {width}, height: {height} }}\n  tile_size: {{ width: {tile_width}, height: {tile_height} }}\n  columns: {columns}\n  rows: {rows}\n  tile_count: {tile_count}\n  margin: {{ x: {margin_x}, y: {margin_y} }}\n  spacing: {{ x: {spacing_x}, y: {spacing_y} }}\n  indexing: row_major_0_based\n\ndefaults:\n  collision: solid\n  damageable: true\n\ntiles: {{}}\n"
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
                "kind: sprite-sheet-2d\nschema_version: 1\nid: {asset_id}\nlabel: {label}\n\nsource:\n{source_block}\n\natlas:\n  frame_size: {{ width: {tile_width}, height: {tile_height} }}\n  columns: {columns}\n  rows: {rows}\n  frame_count: {frame_count}\n  margin: {{ x: {margin_x}, y: {margin_y} }}\n  spacing: {{ x: {spacing_x}, y: {spacing_y} }}\n  fps: {fps}\n  looping: true\n\nanimations: {{}}\n"
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
    if normalized.starts_with("assets/") {
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

    use super::{create_asset_descriptor, scan_asset_registry};
    use crate::asset_registry::dto::{AssetStatusDto, CreateAssetDescriptorRequestDto};

    #[test]
    fn scans_managed_raw_orphan_and_missing_sources() {
        let root = test_root("scan");
        write_png(root.join("assets/raw/images/dirt.png"), 16, 16);
        write_png(root.join("assets/raw/images/unused.png"), 16, 16);
        fs::write(
            root.join("assets/tilesets/dirt.tileset.yml"),
            r#"
kind: tileset-2d
schema_version: 1
id: dirt
label: Dirt
source:
  file: ../raw/images/dirt.png
atlas:
  image_size: { width: 16, height: 16 }
  tile_size: { width: 16, height: 16 }
  columns: 1
  rows: 1
  tile_count: 1
"#,
        )
        .unwrap();
        fs::write(
            root.join("assets/images/missing.image.yml"),
            r#"
kind: image-2d
schema_version: 1
id: missing
source:
  file: ../raw/images/missing.png
"#,
        )
        .unwrap();

        let registry = scan_asset_registry("session-1", "ink-wars", &root).unwrap();

        assert_eq!(registry.managed_assets.len(), 2);
        assert!(
            registry
                .raw_files
                .iter()
                .any(|file| file.relative_path == "assets/raw/images/unused.png" && file.orphan)
        );
        assert!(registry.managed_assets.iter().any(|asset| {
            asset.asset_key == "ink-wars/images/missing"
                && matches!(asset.status, AssetStatusDto::MissingSource)
        }));
    }

    #[test]
    fn creates_descriptor_from_raw_file() {
        let root = test_root("create");
        write_png(root.join("assets/raw/images/paper.png"), 32, 16);

        let asset = create_asset_descriptor(
            "ink-wars",
            &root,
            CreateAssetDescriptorRequestDto {
                raw_file_path: "assets/raw/images/paper.png".to_owned(),
                kind: "image".to_owned(),
                asset_id: "paper".to_owned(),
                import_options: None,
            },
        )
        .unwrap();

        assert_eq!(asset.asset_key, "ink-wars/images/paper");
        assert_eq!(
            asset.descriptor_relative_path,
            "assets/images/paper.image.yml"
        );
        let written = fs::read_to_string(root.join("assets/images/paper.image.yml")).unwrap();
        assert!(written.contains("kind: image-2d"));
        assert!(written.contains("file: ../raw/images/paper.png"));
    }

    fn test_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("amigo-editor-asset-registry-test-{name}-{stamp}"));
        fs::create_dir_all(root.join("assets/raw/images")).unwrap();
        fs::create_dir_all(root.join("assets/images")).unwrap();
        fs::create_dir_all(root.join("assets/tilesets")).unwrap();
        root
    }

    fn write_png(path: PathBuf, width: u32, height: u32) {
        let image =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        image.save(path).unwrap();
    }
}
