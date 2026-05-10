use crate::dto::{
    EditorModDetailsDto, EditorProjectFileDto, EditorProjectStructureNodeDto, EditorSceneSummaryDto,
};

/// @codemap P1 editor.project_structure.root
pub fn project_structure_root(
    details: &EditorModDetailsDto,
    file_root: &EditorProjectFileDto,
) -> EditorProjectStructureNodeDto {
    let summary = &details.summary.content_summary;
    let diagnostics_count = details.summary.diagnostics.len()
        + details
            .scenes
            .iter()
            .map(|scene| scene.diagnostics.len())
            .sum::<usize>();
    let script_files = flatten_project_files(file_root)
        .into_iter()
        .filter(|file| {
            file.kind == "script" && !scene_owns_script(&details.scenes, &file.relative_path)
        })
        .cloned()
        .collect::<Vec<_>>();
    let package_files = flatten_project_files(file_root)
        .into_iter()
        .filter(|file| file.relative_path.starts_with("packages/"))
        .cloned()
        .collect::<Vec<_>>();

    node(ProjectStructureNodeInput {
        id: format!("mod:{}", details.summary.id),
        label: details.summary.id.clone(),
        kind: "modRoot",
        icon: "Mod",
        status: Some(project_status_for_editor_status(&format!(
            "{:?}",
            details.summary.status
        ))),
        count: Some(summary.total_files),
        path: Some(details.summary.root_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: None,
        scene: None,
        children: vec![
            node(ProjectStructureNodeInput {
                id: "overview".to_owned(),
                label: "Overview".to_owned(),
                kind: "overview",
                icon: "Info",
                status: Some(project_status_for_editor_status(&format!(
                    "{:?}",
                    details.summary.status
                ))),
                count: None,
                path: None,
                expected_path: None,
                exists: true,
                empty: false,
                ghost: false,
                file: None,
                scene: None,
                children: Vec::new(),
            }),
            manifest_node(file_root, details),
            group_node(
                "scenes",
                "Sc",
                details.scenes.len(),
                root_child_exists(file_root, "scenes"),
                details
                    .scenes
                    .iter()
                    .map(|scene| scene_structure_node(scene, file_root))
                    .collect(),
            ),
            group_node(
                "raw",
                "Raw",
                summary.textures + summary.audio + summary.fonts,
                root_child_exists(file_root, "raw"),
                files_under(file_root, "raw")
                    .into_iter()
                    .take(48)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "spritesheets",
                "Grid",
                summary.spritesheets + summary.tilesets + summary.tilemaps,
                root_child_exists(file_root, "spritesheets"),
                files_under(file_root, "spritesheets")
                    .into_iter()
                    .filter(|file| {
                        matches!(file.kind.as_str(), "spritesheet" | "tileset" | "tilemap")
                    })
                    .take(64)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "audio",
                "Aud",
                summary.audio,
                root_child_exists(file_root, "audio"),
                files_under(file_root, "audio")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "fonts",
                "Type",
                summary.fonts,
                root_child_exists(file_root, "fonts"),
                files_under(file_root, "fonts")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "scripts",
                "Rh",
                script_files.len(),
                root_child_exists(file_root, "scripts"),
                script_files
                    .into_iter()
                    .take(24)
                    .map(|file| file_structure_node(file, "scriptFile"))
                    .collect(),
            ),
            group_node(
                "packages",
                "Pkg",
                summary.packages,
                root_child_exists(file_root, "packages"),
                package_files
                    .into_iter()
                    .take(24)
                    .map(|file| file_structure_node(file, "scriptPackage"))
                    .collect(),
            ),
            group_node(
                "data",
                "Data",
                files_under(file_root, "data").len(),
                root_child_exists(file_root, "data"),
                files_under(file_root, "data")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "docs",
                "Doc",
                files_under(file_root, "docs").len(),
                root_child_exists(file_root, "docs"),
                files_under(file_root, "docs")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "custom",
                "Ext",
                files_under(file_root, "custom").len(),
                root_child_exists(file_root, "custom"),
                files_under(file_root, "custom")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            virtual_node(
                "capabilities",
                "Capabilities",
                "Plug",
                details.summary.capabilities.len(),
                "ok",
            ),
            virtual_node(
                "dependencies",
                "Dependencies",
                "Link",
                details.summary.dependencies.len(),
                if details.summary.missing_dependencies.is_empty() {
                    "ok"
                } else {
                    "warn"
                },
            ),
            virtual_node(
                "diagnostics",
                "Diagnostics",
                "Diag",
                diagnostics_count,
                if diagnostics_count == 0 { "ok" } else { "warn" },
            ),
        ],
    })
}

struct ProjectStructureNodeInput {
    id: String,
    label: String,
    kind: &'static str,
    icon: &'static str,
    status: Option<String>,
    count: Option<usize>,
    path: Option<String>,
    expected_path: Option<String>,
    exists: bool,
    empty: bool,
    ghost: bool,
    file: Option<EditorProjectFileDto>,
    scene: Option<EditorSceneSummaryDto>,
    children: Vec<EditorProjectStructureNodeDto>,
}

fn node(input: ProjectStructureNodeInput) -> EditorProjectStructureNodeDto {
    EditorProjectStructureNodeDto {
        id: input.id,
        label: input.label,
        kind: input.kind.to_owned(),
        icon: input.icon.to_owned(),
        status: input.status,
        count: input.count,
        path: input.path,
        expected_path: input.expected_path,
        exists: input.exists,
        empty: input.empty,
        ghost: input.ghost,
        file: input.file,
        scene: input.scene,
        children: input.children,
    }
}

fn manifest_node(
    file_root: &EditorProjectFileDto,
    details: &EditorModDetailsDto,
) -> EditorProjectStructureNodeDto {
    let manifest = find_project_file(file_root, "mod.toml").cloned();
    node(ProjectStructureNodeInput {
        id: "manifest:mod.toml".to_owned(),
        label: "mod.toml".to_owned(),
        kind: "manifest",
        icon: "Toml",
        status: Some(if manifest.is_some() {
            project_status_for_editor_status(&format!("{:?}", details.summary.status))
        } else {
            "error".to_owned()
        }),
        count: None,
        path: manifest.as_ref().map(|file| file.relative_path.clone()),
        expected_path: Some("mod.toml".to_owned()),
        exists: manifest.is_some(),
        empty: false,
        ghost: manifest.is_none(),
        file: manifest,
        scene: None,
        children: Vec::new(),
    })
}

fn group_node(
    label: &str,
    icon: &'static str,
    count: usize,
    exists: bool,
    children: Vec<EditorProjectStructureNodeDto>,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("group:{label}"),
        label: label.to_owned(),
        kind: if exists { "folder" } else { "expectedFolder" },
        icon,
        status: Some(
            if exists {
                if count == 0 { "empty" } else { "ok" }
            } else {
                "missing"
            }
            .to_owned(),
        ),
        count: Some(count),
        path: if exists { Some(label.to_owned()) } else { None },
        expected_path: Some(format!("{label}/")),
        exists,
        empty: exists && count == 0,
        ghost: !exists,
        file: None,
        scene: None,
        children,
    })
}

fn virtual_node(
    id: &'static str,
    label: &str,
    icon: &'static str,
    count: usize,
    status: &str,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("virtual:{id}"),
        label: label.to_owned(),
        kind: id,
        icon,
        status: Some(status.to_owned()),
        count: Some(count),
        path: None,
        expected_path: None,
        exists: true,
        empty: count == 0,
        ghost: false,
        file: None,
        scene: None,
        children: Vec::new(),
    })
}

fn scene_structure_node(
    scene: &EditorSceneSummaryDto,
    file_root: &EditorProjectFileDto,
) -> EditorProjectStructureNodeDto {
    let document_path = relative_project_path(&scene.document_path);
    let script_path = relative_project_path(&scene.script_path);
    let document = find_project_file(file_root, &document_path).cloned();
    let script = find_project_file(file_root, &script_path).cloned();
    let status = project_status_for_editor_status(&format!("{:?}", scene.status));

    node(ProjectStructureNodeInput {
        id: format!("scene:{}", scene.id),
        label: if scene.label.is_empty() {
            scene.id.clone()
        } else {
            scene.label.clone()
        },
        kind: "scene",
        icon: "Play",
        status: Some(if status == "valid" {
            "ready".to_owned()
        } else {
            status
        }),
        count: Some(2),
        path: Some(scene.path.clone()),
        expected_path: None,
        exists: document.is_some(),
        empty: false,
        ghost: false,
        file: None,
        scene: Some(scene.clone()),
        children: vec![
            scene_file_node(
                "sceneDocument",
                format!("scene-doc:{}", scene.id),
                "scene.yml",
                "Yml",
                document_path,
                document,
            ),
            scene_file_node(
                "sceneScript",
                format!("scene-script:{}", scene.id),
                "scene.rhai",
                "Rh",
                script_path,
                script,
            ),
        ],
    })
}

fn scene_file_node(
    kind: &'static str,
    id: String,
    label: &str,
    icon: &'static str,
    expected_path: String,
    file: Option<EditorProjectFileDto>,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id,
        label: label.to_owned(),
        kind,
        icon,
        status: Some(if file.is_some() { "ok" } else { "missing" }.to_owned()),
        count: None,
        path: file.as_ref().map(|file| file.relative_path.clone()),
        expected_path: Some(expected_path),
        exists: file.is_some(),
        empty: false,
        ghost: file.is_none(),
        file,
        scene: None,
        children: Vec::new(),
    })
}

fn file_structure_node(
    file: EditorProjectFileDto,
    kind: &'static str,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("{kind}:{}", file.relative_path),
        label: file.name.clone(),
        kind,
        icon: project_file_icon(&file),
        status: Some("ok".to_owned()),
        count: None,
        path: Some(file.relative_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: Some(file),
        scene: None,
        children: Vec::new(),
    })
}

fn asset_resource_node(file: EditorProjectFileDto) -> EditorProjectStructureNodeDto {
    let label = asset_display_label(&file);
    node(ProjectStructureNodeInput {
        id: format!("assetResource:{}", file.relative_path),
        label,
        kind: "assetResource",
        icon: project_file_icon(&file),
        status: Some("ok".to_owned()),
        count: None,
        path: Some(file.relative_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: Some(file),
        scene: None,
        children: Vec::new(),
    })
}

fn flatten_project_files(root: &EditorProjectFileDto) -> Vec<&EditorProjectFileDto> {
    root.children
        .iter()
        .flat_map(|child| {
            let mut files = vec![child];
            files.extend(flatten_project_files(child));
            files
        })
        .filter(|file| !file.is_dir)
        .collect()
}

fn find_project_file<'a>(
    root: &'a EditorProjectFileDto,
    relative_path: &str,
) -> Option<&'a EditorProjectFileDto> {
    if root.relative_path == relative_path {
        return Some(root);
    }
    root.children
        .iter()
        .find_map(|child| find_project_file(child, relative_path))
}

fn root_child_exists(root: &EditorProjectFileDto, relative_path: &str) -> bool {
    find_project_file(root, relative_path).is_some()
}

fn files_under(root: &EditorProjectFileDto, relative_path: &str) -> Vec<EditorProjectFileDto> {
    let prefix = format!("{}/", relative_path.trim_end_matches('/'));
    flatten_project_files(root)
        .into_iter()
        .filter(|file| {
            file.relative_path == relative_path || file.relative_path.starts_with(&prefix)
        })
        .cloned()
        .collect()
}

fn relative_project_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    for prefix in [
        "scenes/",
        "raw/",
        "spritesheets/",
        "audio/",
        "fonts/",
        "scripts/",
        "data/",
        "docs/",
        "custom/",
        "packages/",
    ] {
        if let Some(index) = normalized.find(prefix) {
            return normalized[index..].to_owned();
        }
    }
    normalized
}

fn project_status_for_editor_status(status: &str) -> String {
    match status {
        "Valid" => "valid",
        "Warning" | "MissingDependency" => "warn",
        "Error" | "InvalidManifest" | "MissingSceneFile" | "PreviewFailed" => "error",
        _ => "ok",
    }
    .to_owned()
}

fn project_file_icon(file: &EditorProjectFileDto) -> &'static str {
    match file.kind.as_str() {
        "manifest" => "Toml",
        "sceneDocument" => "Yml",
        "script" => "Rh",
        "imageAsset" | "rawImage" | "texture" => "Img",
        "spritesheet" => "Grid",
        "audio" | "rawAudio" => "Aud",
        "font" | "rawFont" => "Type",
        "tilemap" => "Map",
        "tileset" => "Tile",
        "particle" => "Pt",
        "material" => "Mat",
        "ui" => "Ui",
        _ => "F",
    }
}

fn asset_display_label(file: &EditorProjectFileDto) -> String {
    let name = file.name.as_str();
    for suffix in [
        ".image.yml",
        ".image.yaml",
        ".sprite.yml",
        ".sprite.yaml",
        ".atlas.yml",
        ".atlas.yaml",
        ".tileset.yml",
        ".tileset.yaml",
        ".tile-ruleset.yml",
        ".tile-ruleset.yaml",
        ".tilemap.yml",
        ".tilemap.yaml",
        ".font.yml",
        ".font.yaml",
        ".audio.yml",
        ".audio.yaml",
        ".particle.yml",
        ".particle.yaml",
        ".material.yml",
        ".material.yaml",
        ".ui.yml",
        ".ui.yaml",
    ] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            return stripped.to_owned();
        }
    }
    name.to_owned()
}

fn scene_owns_script(scenes: &[EditorSceneSummaryDto], relative_path: &str) -> bool {
    scenes
        .iter()
        .any(|scene| relative_project_path(&scene.script_path) == relative_path)
}
