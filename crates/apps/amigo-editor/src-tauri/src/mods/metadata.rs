use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use amigo_modding::{DiscoveredMod, ModSceneManifest};

use crate::cache::project_id::project_cache_id_for_root;
use crate::dto::{
    DiagnosticLevel, EditorContentSummaryDto, EditorDiagnosticDto, EditorModDetailsDto,
    EditorModSummaryDto, EditorSceneSummaryDto, EditorStatus, PreviewStatus,
};

const PREVIEW_FPS: u32 = 5;

pub fn mod_summary(
    discovered: &DiscoveredMod,
    discovered_ids: &BTreeSet<String>,
) -> EditorModSummaryDto {
    let manifest = &discovered.manifest;
    let missing_dependencies = manifest
        .dependencies
        .iter()
        .filter(|dependency| !discovered_ids.contains(*dependency))
        .cloned()
        .collect::<Vec<_>>();
    let scene_summaries = scene_summaries(discovered);
    let mut diagnostics = Vec::new();

    for dependency in &missing_dependencies {
        diagnostics.push(EditorDiagnosticDto {
            level: DiagnosticLevel::Error,
            code: "missing_dependency".to_owned(),
            message: format!("Dependency `{dependency}` is not present in mods root."),
            path: Some(discovered.root_path.display().to_string()),
        });
    }

    for scene in &scene_summaries {
        diagnostics.extend(scene.diagnostics.clone());
    }

    let status = if !missing_dependencies.is_empty() {
        EditorStatus::MissingDependency
    } else if scene_summaries
        .iter()
        .any(|scene| scene.status == EditorStatus::MissingSceneFile)
    {
        EditorStatus::MissingSceneFile
    } else if diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Warning))
    {
        EditorStatus::Warning
    } else {
        EditorStatus::Valid
    };

    EditorModSummaryDto {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        authors: manifest.authors.clone(),
        root_path: discovered.root_path.display().to_string(),
        dependencies: manifest.dependencies.clone(),
        missing_dependencies,
        capabilities: manifest.capabilities.clone(),
        scene_count: manifest.scenes.len(),
        visible_scene_count: manifest
            .scenes
            .iter()
            .filter(|scene| scene.is_launcher_visible())
            .count(),
        project_cache_id: project_cache_id_for_root(&discovered.root_path),
        status,
        diagnostics,
        last_modified: last_modified(&discovered.root_path.join("mod.toml")),
        preview_status: PreviewStatus::Missing,
        content_summary: content_summary(discovered),
    }
}

pub fn mod_details(
    discovered: &DiscoveredMod,
    discovered_ids: &BTreeSet<String>,
) -> EditorModDetailsDto {
    EditorModDetailsDto {
        summary: mod_summary(discovered, discovered_ids),
        scenes: scene_summaries(discovered),
    }
}

pub fn scene_summaries(discovered: &DiscoveredMod) -> Vec<EditorSceneSummaryDto> {
    discovered
        .manifest
        .scenes
        .iter()
        .map(|scene| scene_summary(discovered, scene))
        .collect()
}

fn scene_summary(discovered: &DiscoveredMod, scene: &ModSceneManifest) -> EditorSceneSummaryDto {
    let document_path = scene.document_path(&discovered.root_path);
    let script_path = scene.script_path(&discovered.root_path);
    let project_cache_id = project_cache_id_for_root(&discovered.root_path);
    let mut diagnostics = Vec::new();

    if !document_path.is_file() {
        diagnostics.push(EditorDiagnosticDto {
            level: DiagnosticLevel::Error,
            code: "missing_scene_document".to_owned(),
            message: format!("Scene document `{}` is missing.", document_path.display()),
            path: Some(document_path.display().to_string()),
        });
    }

    let status = if diagnostics.is_empty() {
        EditorStatus::Valid
    } else {
        EditorStatus::MissingSceneFile
    };

    EditorSceneSummaryDto {
        id: scene.id.clone(),
        label: if scene.label.is_empty() {
            scene.id.clone()
        } else {
            scene.label.clone()
        },
        description: scene.description.clone(),
        path: scene.root_path(&discovered.root_path).display().to_string(),
        document_path: document_path.display().to_string(),
        script_path: script_path.display().to_string(),
        launcher_visible: scene.launcher_visible,
        status,
        preview_cache_key: format!(
            "{project_cache_id}:{}:engine-slideshow-preview-v1",
            scene.id
        ),
        preview_image_url: None,
        preview_fps: PREVIEW_FPS,
        diagnostics,
    }
}

fn last_modified(path: &Path) -> Option<String> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let since_epoch = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(format!("{}", since_epoch.as_secs()))
}

fn content_summary(discovered: &DiscoveredMod) -> EditorContentSummaryDto {
    let mut summary = EditorContentSummaryDto {
        scenes: discovered.manifest.scenes.len(),
        ..EditorContentSummaryDto::default()
    };
    collect_content_summary(&discovered.root_path, &discovered.root_path, &mut summary);
    summary
}

fn collect_content_summary(mod_root: &Path, current: &Path, summary: &mut EditorContentSummaryDto) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_content_summary(mod_root, &path, summary);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        summary.total_files += 1;
        classify_file(mod_root, &path, summary);
    }
}

fn classify_file(mod_root: &Path, path: &Path, summary: &mut EditorContentSummaryDto) {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let relative_parts = path
        .strip_prefix(mod_root)
        .ok()
        .map(|relative| {
            relative
                .components()
                .filter_map(|component| component.as_os_str().to_str())
                .map(|part| part.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let top_level = relative_parts.first().map(String::as_str);
    let asset_group = if top_level == Some("assets") {
        relative_parts.get(1).map(String::as_str)
    } else {
        top_level
    };

    if file_name == "scene.yml" || file_name == "scene.yaml" {
        summary.scene_yaml += 1;
        return;
    }

    if file_name == "mod.toml" {
        summary.packages += 1;
        return;
    }

    if matches!(top_level, Some("docs" | "sources")) {
        return;
    }

    match asset_group {
        Some("raw") => {
            return;
        }
        Some("images") => {
            if file_name.ends_with(".image.yml") || file_name.ends_with(".image.yaml") {
                summary.textures += 1;
                return;
            }
        }
        Some("sprites") => {
            if file_name.ends_with(".sprite.yml")
                || file_name.ends_with(".sprite.yaml")
                || file_name.ends_with(".atlas.yml")
                || file_name.ends_with(".atlas.yaml")
            {
                summary.spritesheets += 1;
                return;
            }
        }
        Some("tilesets") => {
            if file_name.ends_with(".tileset.yml")
                || file_name.ends_with(".tileset.yaml")
                || file_name.ends_with(".tile-ruleset.yml")
                || file_name.ends_with(".tile-ruleset.yaml")
            {
                summary.tilesets += 1;
                return;
            }
        }
        Some("tilemaps") => {
            if file_name.ends_with(".tilemap.yml") || file_name.ends_with(".tilemap.yaml") {
                summary.tilemaps += 1;
                return;
            }
        }
        Some("audio") => {
            if file_name.ends_with(".audio.yml") || file_name.ends_with(".audio.yaml") {
                summary.audio += 1;
                return;
            }
        }
        Some("fonts") => {
            if file_name.ends_with(".font.yml") || file_name.ends_with(".font.yaml") {
                summary.fonts += 1;
                return;
            }
        }
        _ => {}
    }

    if extension == "toml" {
        summary.packages += 1;
        return;
    }

    if extension == "rhai" {
        summary.scripts += 1;
        return;
    }

    if matches!(extension.as_str(), "yml" | "yaml") {
        if file_name.ends_with(".tileset.yml")
            || file_name.ends_with(".tileset.yaml")
            || file_name.ends_with(".tile-ruleset.yml")
            || file_name.ends_with(".tile-ruleset.yaml")
        {
            summary.tilesets += 1;
        } else if file_name.ends_with(".tilemap.yml") || file_name.ends_with(".tilemap.yaml") {
            summary.tilemaps += 1;
        } else if file_name.ends_with(".image.yml") || file_name.ends_with(".image.yaml") {
            summary.textures += 1;
        } else if file_name.ends_with(".font.yml") || file_name.ends_with(".font.yaml") {
            summary.fonts += 1;
        } else if file_name.ends_with(".audio.yml") || file_name.ends_with(".audio.yaml") {
            summary.audio += 1;
        } else {
            summary.unknown_files += 1;
        }
        return;
    }

    summary.unknown_files += 1;
}
