use std::fs;
use std::path::{Path, PathBuf};

use rfd::FileDialog;
use tauri::AppHandle;

use crate::dto::{
    AddItemKindDto, CreateProjectItemRequestDto, CreateProjectItemResultDto, DiagnosticLevel,
    EditorDiagnosticDto,
};
use crate::events::bus;
use crate::mods::discovery::discover_editor_mods;

fn slugify(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .replace("--", "-")
}

fn resolve_mod_root(mod_id: &str) -> Result<PathBuf, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    discovered_mod.root_path.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            discovered_mod.root_path.display()
        )
    })
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let candidate = root.join(relative.trim_matches('/'));
    let parent = candidate
        .parent()
        .ok_or_else(|| format!("invalid target path `{relative}`"))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create parent directory `{}`: {error}",
            parent.display()
        )
    })?;
    let canonical_parent = parent.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize parent directory `{}`: {error}",
            parent.display()
        )
    })?;
    if !canonical_parent.starts_with(root) {
        return Err(format!("target path `{relative}` escapes mod root"));
    }
    Ok(candidate)
}

fn write_new_file(path: &Path, content: &str, overwrite: bool) -> Result<(), String> {
    if path.exists() && !overwrite {
        return Err(format!("project file `{}` already exists", path.display()));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("failed to create directory `{}`: {error}", parent.display())
        })?;
    }
    fs::write(path, content.as_bytes())
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

fn yaml_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn render_scene_yaml(scene_id: &str, label: &str) -> String {
    format!(
        "version: 1
scene:
  id: {scene_id}
  label: {label}
  description: ''

entities: []
",
        scene_id = scene_id,
        label = label
    )
}

fn render_scene_rhai(label: &str) -> String {
    format!(
        "fn on_enter() {{
    world.dev.log(\"{label} loaded.\");
}}

fn update(dt) {{}}
"
    )
}

fn render_ui_theme_yml(item_id: &str, label: &str) -> String {
    format!(
        "kind: ui-theme
schema_version: 1
id: {item_id}
label: {label}
palette:
  background: '#050812FF'
  surface: '#101827EE'
  surface_alt: '#172033EE'
  text: '#EAF6FFFF'
  text_muted: '#7D8796FF'
  border: '#2A6F9EFF'
  accent: '#39D7FFFF'
  accent_text: '#001018FF'
  danger: '#FF4D6DFF'
  warning: '#FFB000FF'
  success: '#5CFF9CFF'
",
        item_id = item_id,
        label = yaml_single_quote(label),
    )
}

fn render_ui_document_yml(item_id: &str, label: &str) -> String {
    format!(
        "kind: ui-document
schema_version: 1
id: {item_id}
label: {label}
target:
  type: screen-space
  layer: hud
  viewport:
    width: 1280.0
    height: 720.0
    scaling: fit
root:
  type: column
  id: root
  style_class: root
  style:
    left: 0.0
    top: 0.0
    width: 1280.0
    height: 720.0
    padding: 32.0
    gap: 12.0
    background: '#00000000'
  children: []
",
        item_id = item_id,
        label = yaml_single_quote(label),
    )
}

fn render_ui_main_menu_yml(item_id: &str, label: &str) -> String {
    format!(
        "kind: ui-main-menu
schema_version: 1
id: {item_id}
label: {label}
target:
  type: screen-space
  layer: menu
  viewport:
    width: 1280.0
    height: 720.0
    scaling: fit
root:
  type: column
  id: root
  style_class: root
  style:
    left: 0.0
    top: 0.0
    width: 1280.0
    height: 720.0
    padding: 0.0
    gap: 14.0
    background: '#050812FF'
  children:
    - type: spacer
      id: top-space
      style:
        width: 1280.0
        height: 76.0
    - type: text
      id: case-file
      text: 'NEW PROJECT MENU'
      font: fonts/debug-ui
      style_class: text-muted
      style:
        width: 1280.0
        height: 28.0
        font_size: 15.0
        color: '#7D8796FF'
        align: center
    - type: text
      id: title
      text: {title}
      font: fonts/debug-ui
      style_class: text-title
      style:
        width: 1280.0
        height: 74.0
        font_size: 58.0
        color: '#EAF6FFFF'
        align: center
    - type: text
      id: subtitle
      text: 'New project menu'
      font: fonts/debug-ui
      style_class: text-muted
      style:
        width: 1280.0
        height: 34.0
        font_size: 17.0
        color: '#89A2B7FF'
        align: center
    - type: spacer
      id: title-menu-gap
      style:
        width: 1280.0
        height: 26.0
    - type: panel
      id: menu-card
      style_class: panel
      style:
        left: 420.0
        width: 440.0
        height: 246.0
        padding: 20.0
        gap: 12.0
        background: '#101827EE'
        border_color: '#2A6F9EFF'
        border_width: 1.0
        border_radius: 10.0
      children:
        - type: text
          id: menu-title
          text: 'MAIN MENU'
          font: fonts/debug-ui
          style_class: text-muted
          style:
            width: 400.0
            height: 28.0
            font_size: 15.0
            color: '#7D8796FF'
            align: center
        - type: button
          id: start
          text: 'START'
          font: fonts/debug-ui
          style_class: button-secondary
          style:
            width: 400.0
            height: 48.0
            font_size: 20.0
            color: '#39D7FFFF'
            background: '#172033EE'
            border_color: '#2A6F9EFF'
            border_width: 1.0
            border_radius: 8.0
            align: center
          on_click:
            event: menu.start
        - type: button
          id: options
          text: 'OPTIONS'
          font: fonts/debug-ui
          style_class: button-secondary
          style:
            width: 400.0
            height: 48.0
            font_size: 20.0
            color: '#7D8796FF'
            background: '#172033EE'
            border_color: '#2A6F9EFF'
            border_width: 1.0
            border_radius: 8.0
            align: center
          on_click:
            event: menu.options
        - type: button
          id: quit
          text: 'QUIT'
          font: fonts/debug-ui
          style_class: button-secondary
          style:
            width: 400.0
            height: 48.0
            font_size: 20.0
            color: '#7D8796FF'
            background: '#172033EE'
            border_color: '#2A6F9EFF'
            border_width: 1.0
            border_radius: 8.0
            align: center
          on_click:
            event: menu.quit
    - type: text
      id: status
      text: 'READY'
      font: fonts/debug-ui
      style_class: text-muted
      style:
        width: 1280.0
        height: 32.0
        font_size: 15.0
        color: '#7D8796FF'
        align: center
",
        item_id = item_id,
        label = yaml_single_quote(label),
        title = yaml_single_quote(label),
    )
}

fn append_scene_to_manifest(
    root: &Path,
    scene_id: &str,
    label: &str,
    launcher_visible: bool,
) -> Result<(), String> {
    let path = root.join("mod.toml");
    let mut content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    if content.contains(&format!("id = \"{scene_id}\"")) && content.contains("[[scenes]]") {
        return Ok(());
    }
    let block = format!(
        "\n[[scenes]]\nid = \"{scene_id}\"\nlabel = \"{label}\"\ndescription = \"\"\npath = \"scenes/{scene_id}\"\nlauncher_visible = {launcher_visible}\n"
    );
    content.push_str(&block);
    fs::write(&path, content.as_bytes())
        .map_err(|error| format!("failed to update `{}`: {error}", path.display()))
}

pub fn create_project_item(
    app: AppHandle,
    mod_id: String,
    request: CreateProjectItemRequestDto,
) -> Result<CreateProjectItemResultDto, String> {
    let root = resolve_mod_root(&mod_id)?;
    let item_id = slugify(&request.item_id);
    if item_id.is_empty() {
        return Err("item id is required".to_owned());
    }
    let overwrite = request
        .options
        .as_ref()
        .and_then(|options| options.overwrite)
        .unwrap_or(false);
    let mut created_files = Vec::new();
    let mut updated_files = Vec::new();
    let selected_file_path: Option<String>;

    match request.item_kind {
        AddItemKindDto::Scene => {
            let label = request
                .label
                .clone()
                .unwrap_or_else(|| "New Scene".to_owned());
            let scene_yml_rel = format!("scenes/{item_id}/scene.yml");
            let scene_yml = safe_join(&root, &scene_yml_rel)?;
            write_new_file(&scene_yml, &render_scene_yaml(&item_id, &label), overwrite)?;
            created_files.push(scene_yml_rel.clone());
            selected_file_path = Some(scene_yml_rel.clone());

            let create_script = request
                .options
                .as_ref()
                .and_then(|options| options.create_script)
                .unwrap_or(true);
            if create_script {
                let scene_rhai_rel = format!("scenes/{item_id}/scene.rhai");
                let scene_rhai = safe_join(&root, &scene_rhai_rel)?;
                write_new_file(&scene_rhai, &render_scene_rhai(&label), overwrite)?;
                created_files.push(scene_rhai_rel);
            }

            let launcher_visible = request
                .options
                .as_ref()
                .and_then(|options| options.launcher_visible)
                .unwrap_or(false);
            append_scene_to_manifest(&root, &item_id, &label, launcher_visible)?;
            updated_files.push("mod.toml".to_owned());
        }
        AddItemKindDto::Script => {
            let folder = request
                .target_folder
                .clone()
                .unwrap_or_else(|| "scripts".to_owned());
            let rel = format!("{}/{}.rhai", folder.trim_matches('/'), item_id);
            let path = safe_join(&root, &rel)?;
            write_new_file(&path, "fn update(dt) {}\n", overwrite)?;
            created_files.push(rel.clone());
            selected_file_path = Some(rel);
        }
        AddItemKindDto::Folder => {
            let base = request.target_folder.clone().unwrap_or_default();
            let rel = if base.trim().is_empty() {
                item_id.clone()
            } else {
                format!("{}/{}", base.trim_matches('/'), item_id)
            };
            let path = safe_join(&root, &format!("{rel}/.keep"))?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!("failed to create folder `{}`: {error}", parent.display())
                })?;
            }
            selected_file_path = Some(rel);
        }
        AddItemKindDto::Font => {
            let rel = format!("fonts/{item_id}/font.yml");
            let path = safe_join(&root, &rel)?;
            let content = format!(
                "kind: font-2d
schema_version: 1
id: {item_id}
label: '{}'
format: truetype

source:
  file: {item_id}.ttf

glyphs:
  preset: console-latin-ext
  extra: \"→←↑↓✓✗•\"

metrics:
  default_size: 12.0
  line_height: 15.0
  letter_spacing: 0.0
  tab_width: 4

fallback:
  missing_glyph: \"?\"
",
                request
                    .label
                    .clone()
                    .unwrap_or_else(|| format!("{item_id} font"))
            );
            write_new_file(&path, &content, overwrite)?;
            created_files.push(rel.clone());
            selected_file_path = Some(rel);
        }
        AddItemKindDto::UiTheme => {
            let label = request
                .label
                .clone()
                .unwrap_or_else(|| format!("{item_id} theme"));
            let rel = format!("ui/themes/{item_id}.yml");
            let path = safe_join(&root, &rel)?;
            write_new_file(&path, &render_ui_theme_yml(&item_id, &label), overwrite)?;
            created_files.push(rel.clone());
            selected_file_path = Some(rel);
        }
        AddItemKindDto::UiDocument => {
            let label = request
                .label
                .clone()
                .unwrap_or_else(|| format!("{item_id} UI document"));
            let rel = format!("ui/documents/{item_id}.yml");
            let path = safe_join(&root, &rel)?;
            write_new_file(&path, &render_ui_document_yml(&item_id, &label), overwrite)?;
            created_files.push(rel.clone());
            selected_file_path = Some(rel);
        }
        AddItemKindDto::UiMainMenu => {
            let label = request
                .label
                .clone()
                .unwrap_or_else(|| "Main Menu".to_owned());
            let rel = format!("ui/menus/{item_id}.yml");
            let path = safe_join(&root, &rel)?;
            write_new_file(&path, &render_ui_main_menu_yml(&item_id, &label), overwrite)?;
            created_files.push(rel.clone());
            selected_file_path = Some(rel);
        }
        AddItemKindDto::RawSource => {
            let source = request
                .source_file_path
                .as_ref()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "source file path is required for raw-source".to_owned())?;
            let source_path = PathBuf::from(&source);
            if !source_path.is_file() {
                return Err(format!("source file `{source}` does not exist"));
            }
            let target_folder = request
                .target_folder
                .clone()
                .unwrap_or_else(|| "raw".to_owned());
            let file_name = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| "source filename is invalid".to_owned())?;
            let rel = format!("{}/{}", target_folder.trim_matches('/'), file_name);
            let target = safe_join(&root, &rel)?;
            if target.exists() && !overwrite {
                return Err(format!("target raw file `{rel}` already exists"));
            }
            fs::copy(&source_path, &target).map_err(|error| {
                format!(
                    "failed to copy `{}` to `{}`: {error}",
                    source_path.display(),
                    target.display()
                )
            })?;
            created_files.push(rel.clone());
            selected_file_path = Some(rel);
        }
        AddItemKindDto::UiHud => {
            return Err("HUD UI templates are not implemented yet".to_owned());
        }
        AddItemKindDto::UiDialog => {
            return Err("Dialog UI templates are not implemented yet".to_owned());
        }
        AddItemKindDto::UiComponent => {
            return Err("UI component templates are not implemented yet".to_owned());
        }
        _ => {
            return Err(format!(
                "item kind `{:?}` is not implemented yet",
                request.item_kind
            ));
        }
    }

    let _ = bus::emit_asset_registry_changed(&app, mod_id);

    Ok(CreateProjectItemResultDto {
        item_kind: request.item_kind,
        item_id,
        created_files,
        updated_files,
        selected_file_path,
        selected_asset_key: None,
        diagnostics: vec![EditorDiagnosticDto {
            level: DiagnosticLevel::Info,
            code: "project_item_created".to_owned(),
            message: "Project item created.".to_owned(),
            path: None,
        }],
    })
}

pub fn pick_project_source_file() -> Result<Option<String>, String> {
    let picked = FileDialog::new()
        .set_title("Choose Source File")
        .pick_file();
    Ok(picked.map(|path| path.display().to_string()))
}
