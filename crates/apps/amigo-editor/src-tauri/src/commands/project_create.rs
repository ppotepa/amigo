use std::fs;
use std::path::Path;

use amigo_modding::ModCatalog;

use crate::dto::{
    CreateModProjectRequestDto, CreateModProjectResultDto, CreateModProjectTypeDto,
    DiagnosticLevel, EditorDiagnosticDto,
};
use crate::mods::discovery::default_mods_root;

const DEFAULT_AUTHOR: &str = "Amigo Engine Team";
const DEFAULT_SCENE_ID: &str = "start";

pub fn create_mod_project(
    request: CreateModProjectRequestDto,
) -> Result<CreateModProjectResultDto, String> {
    let mods_root = default_mods_root();
    create_mod_project_in_root(&mods_root, request)
}

fn create_mod_project_in_root(
    mods_root: &Path,
    request: CreateModProjectRequestDto,
) -> Result<CreateModProjectResultDto, String> {
    validate_request(&request)?;

    if request.project_type != CreateModProjectTypeDto::TwoD {
        return Err("only the 2D project template is available in this editor build".to_owned());
    }

    let project_id = request.project_id.trim().to_owned();
    let project_name = request.project_name.trim().to_owned();

    fs::create_dir_all(mods_root).map_err(|error| {
        format!(
            "failed to create mods root `{}`: {error}",
            mods_root.display()
        )
    })?;

    let canonical_mods_root = mods_root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mods root `{}`: {error}",
            mods_root.display()
        )
    })?;

    if let Ok(discovered) = ModCatalog::discover_unresolved(mods_root) {
        if discovered
            .iter()
            .any(|candidate| candidate.manifest.id == project_id)
        {
            return Err(format!("mod id `{project_id}` already exists"));
        }
    }

    let project_root = mods_root.join(&project_id);
    if project_root.exists() {
        return Err(format!(
            "project folder `{}` already exists",
            project_root.display()
        ));
    }

    create_empty_2d_project(&project_root, &project_id, &project_name)?;

    let canonical_project_root = project_root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize project root `{}`: {error}",
            project_root.display()
        )
    })?;

    if !canonical_project_root.starts_with(&canonical_mods_root) {
        return Err(format!(
            "project root `{}` escapes mods root `{}`",
            canonical_project_root.display(),
            canonical_mods_root.display()
        ));
    }

    validate_generated_project(mods_root, &project_id, &project_root)?;

    Ok(CreateModProjectResultDto {
        mod_id: project_id.clone(),
        root_path: project_root.display().to_string(),
        manifest_path: project_root.join("mod.toml").display().to_string(),
        initial_scene_id: DEFAULT_SCENE_ID.to_owned(),
        created_files: created_files_for(&project_id),
        diagnostics: vec![EditorDiagnosticDto {
            level: DiagnosticLevel::Info,
            code: "project_created".to_owned(),
            message: format!("Created project `{project_name}`."),
            path: Some(project_root.display().to_string()),
        }],
    })
}

fn validate_request(request: &CreateModProjectRequestDto) -> Result<(), String> {
    let project_name = request.project_name.trim();
    let project_id = request.project_id.trim();

    if project_name.is_empty() {
        return Err("project name is required".to_owned());
    }

    if !is_valid_project_id(project_id) {
        return Err(
            "project id must use lowercase letters, numbers, and single dashes between words"
                .to_owned(),
        );
    }

    Ok(())
}

fn is_valid_project_id(value: &str) -> bool {
    if value.is_empty() || value.starts_with('-') || value.ends_with('-') {
        return false;
    }

    let mut previous_was_dash = false;

    for ch in value.chars() {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
        if !valid {
            return false;
        }

        if ch == '-' {
            if previous_was_dash {
                return false;
            }
            previous_was_dash = true;
        } else {
            previous_was_dash = false;
        }
    }

    true
}

fn create_empty_2d_project(
    project_root: &Path,
    project_id: &str,
    project_name: &str,
) -> Result<(), String> {
    let directories = [
        project_root.to_path_buf(),
        project_root.join("scenes").join(DEFAULT_SCENE_ID),
        project_root.join("fonts").join("debug-ui"),
        project_root.join("raw"),
        project_root.join("scripts"),
        project_root.join("data"),
        project_root.join("docs"),
    ];

    for directory in directories {
        fs::create_dir_all(&directory).map_err(|error| {
            format!(
                "failed to create directory `{}`: {error}",
                directory.display()
            )
        })?;
    }

    write_new_file(
        &project_root.join("mod.toml"),
        &render_mod_toml(project_id, project_name),
    )?;

    write_new_file(
        &project_root.join("fonts").join("debug-ui").join("font.yml"),
        &render_font_yml(project_name),
    )?;

    write_new_file(
        &project_root
            .join("scenes")
            .join(DEFAULT_SCENE_ID)
            .join("scene.yml"),
        &render_start_scene_yml(project_id, project_name),
    )?;

    write_new_file(
        &project_root
            .join("scenes")
            .join(DEFAULT_SCENE_ID)
            .join("scene.rhai"),
        &render_start_scene_rhai(project_name),
    )?;

    Ok(())
}

fn write_new_file(path: &Path, content: &str) -> Result<(), String> {
    if path.exists() {
        return Err(format!("project file `{}` already exists", path.display()));
    }

    fs::write(path, content.as_bytes())
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

fn validate_generated_project(
    mods_root: &Path,
    project_id: &str,
    project_root: &Path,
) -> Result<(), String> {
    let discovered = ModCatalog::discover_unresolved(mods_root)
        .map_err(|error| format!("generated mod manifest is invalid: {error}"))?;

    if !discovered
        .iter()
        .any(|candidate| candidate.manifest.id == project_id)
    {
        return Err(format!(
            "generated mod `{project_id}` was not discovered under `{}`",
            mods_root.display()
        ));
    }

    let scene_path = project_root
        .join("scenes")
        .join(DEFAULT_SCENE_ID)
        .join("scene.yml");

    amigo_scene::load_scene_document_from_path(&scene_path).map_err(|error| {
        format!(
            "generated start scene `{}` is invalid: {error}",
            scene_path.display()
        )
    })?;

    Ok(())
}

fn created_files_for(project_id: &str) -> Vec<String> {
    vec![
        format!("mods/{project_id}/mod.toml"),
        format!("mods/{project_id}/scenes/{DEFAULT_SCENE_ID}/scene.yml"),
        format!("mods/{project_id}/scenes/{DEFAULT_SCENE_ID}/scene.rhai"),
        format!("mods/{project_id}/fonts/debug-ui/font.yml"),
    ]
}

fn render_mod_toml(project_id: &str, project_name: &str) -> String {
    format!(
        "id = {project_id}
name = {project_name}
version = \"0.1.0\"
description = \"New Amigo 2D project.\"
authors = [{author}]
dependencies = [\"core\"]
capabilities = [\"rendering_2d\", \"text_2d\", \"vector_2d\"]
launcher_category = [\"2D\", \"Projects\"]

[scripting]
mod_script_mode = \"disabled\"

[[scenes]]
id = \"{DEFAULT_SCENE_ID}\"
label = \"Start\"
description = \"Initial placeholder scene.\"
path = \"scenes/{DEFAULT_SCENE_ID}\"
launcher_visible = true
",
        project_id = toml_string(project_id),
        project_name = toml_string(project_name),
        author = toml_string(DEFAULT_AUTHOR),
    )
}

fn render_font_yml(project_name: &str) -> String {
    format!(
        "kind: font-2d
schema_version: 1
id: debug-ui
label: {label}
format: debug-placeholder
",
        label = yaml_single_quote(&format!("{project_name} Debug UI Placeholder")),
    )
}

fn render_start_scene_yml(project_id: &str, project_name: &str) -> String {
    format!(
        "version: 1
scene:
  id: {scene_id}
  label: Start
  description: {description}
entities:
  - id: camera
    name: {camera_name}
    components:
      - type: Camera2D

  - id: backdrop
    name: {backdrop_name}
    components:
      - type: VectorShape2D
        kind: polygon
        points:
          - x: -2400.0
            y: -1600.0
          - x: 2400.0
            y: -1600.0
          - x: 2400.0
            y: 1600.0
          - x: -2400.0
            y: 1600.0
        stroke_color: '#00000000'
        stroke_width: 0.0
        fill_color: '#050812FF'
        z_index: -100.0

  - id: title
    name: {title_name}
    transform2:
      translation:
        x: 0.0
        y: -60.0
      rotation_radians: 0.0
      scale:
        x: 1.0
        y: 1.0
    components:
      - type: Text2D
        content: {title}
        font: {font}
        bounds:
          x: 720.0
          y: 96.0

  - id: subtitle
    name: {subtitle_name}
    transform2:
      translation:
        x: 0.0
        y: 48.0
      rotation_radians: 0.0
      scale:
        x: 1.0
        y: 1.0
    components:
      - type: Text2D
        content: 'New 2D project placeholder'
        font: {font}
        bounds:
          x: 640.0
          y: 64.0
",
        scene_id = DEFAULT_SCENE_ID,
        description = yaml_single_quote(&format!("Initial placeholder scene for {project_name}.")),
        camera_name = yaml_single_quote(&format!("{project_id}-camera")),
        backdrop_name = yaml_single_quote(&format!("{project_id}-backdrop")),
        title_name = yaml_single_quote(&format!("{project_id}-title")),
        subtitle_name = yaml_single_quote(&format!("{project_id}-subtitle")),
        title = yaml_single_quote(&project_name.to_ascii_uppercase()),
        font = yaml_single_quote(&format!("{project_id}/fonts/debug-ui")),
    )
}

fn render_start_scene_rhai(project_name: &str) -> String {
    format!(
        "fn on_enter() {{
    world.dev.log({message});
}}
",
        message = rhai_string(&format!("{project_name} project loaded.")),
    )
}

fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn yaml_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn rhai_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn temp_mods_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "amigo-editor-project-create-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn request(project_id: &str) -> CreateModProjectRequestDto {
        CreateModProjectRequestDto {
            project_type: CreateModProjectTypeDto::TwoD,
            project_name: "They Are Rotten".to_owned(),
            project_id: project_id.to_owned(),
        }
    }

    #[test]
    fn accepts_slug_project_ids() {
        assert!(is_valid_project_id("they-are-rotten"));
        assert!(is_valid_project_id("project2"));
    }

    #[test]
    fn rejects_invalid_project_ids() {
        assert!(!is_valid_project_id("TheyAreRotten"));
        assert!(!is_valid_project_id("they_are_rotten"));
        assert!(!is_valid_project_id("-they-are-rotten"));
        assert!(!is_valid_project_id("they-are-rotten-"));
        assert!(!is_valid_project_id("they--are-rotten"));
        assert!(!is_valid_project_id("../escape"));
    }

    #[test]
    fn creates_minimal_2d_project() {
        let root = temp_mods_root("creates-minimal-2d-project");

        let result = create_mod_project_in_root(&root, request("they-are-rotten"))
            .expect("project should be created");

        assert_eq!(result.mod_id, "they-are-rotten");
        assert_eq!(result.initial_scene_id, DEFAULT_SCENE_ID);
        assert!(root.join("they-are-rotten/mod.toml").is_file());
        assert!(
            root.join("they-are-rotten/scenes/start/scene.yml")
                .is_file()
        );
        assert!(
            root.join("they-are-rotten/scenes/start/scene.rhai")
                .is_file()
        );
        assert!(
            root.join("they-are-rotten/fonts/debug-ui/font.yml")
                .is_file()
        );

        let discovered = ModCatalog::discover_unresolved(&root).expect("manifest should parse");
        assert!(
            discovered
                .iter()
                .any(|candidate| candidate.manifest.id == "they-are-rotten")
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn blocks_existing_project_folder() {
        let root = temp_mods_root("blocks-existing-folder");
        fs::create_dir_all(root.join("they-are-rotten")).expect("folder should be created");

        let error = create_mod_project_in_root(&root, request("they-are-rotten"))
            .expect_err("existing folder should be rejected");

        assert!(error.contains("already exists"));

        let _ = fs::remove_dir_all(&root);
    }
}
