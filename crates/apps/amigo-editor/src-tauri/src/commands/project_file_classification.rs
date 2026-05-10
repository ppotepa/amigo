use std::path::Path;

/// @codemap P1 editor.project_file.classification
pub fn classify_project_file(path: &Path, is_dir: bool) -> String {
    if is_dir {
        return "directory".to_owned();
    }

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name == "mod.toml" || extension == "toml" {
        "manifest"
    } else if file_name == "package.yml" || file_name == "package.yaml" {
        "scriptPackage"
    } else if file_name == "scene.yml"
        || file_name == "scene.yaml"
        || file_name.ends_with(".scene.yml")
        || file_name.ends_with(".scene.yaml")
    {
        "sceneDocument"
    } else if file_name == "scene.rhai" || file_name.ends_with(".scene.rhai") {
        "sceneScript"
    } else if extension == "rhai" {
        "script"
    } else if file_name.ends_with(".font.yml") || file_name.ends_with(".font.yaml") {
        "font"
    } else if file_name.ends_with(".image.yml") || file_name.ends_with(".image.yaml") {
        "imageAsset"
    } else if file_name == "spritesheet.yml"
        || normalized_path.contains("/spritesheets/") && file_name == "spritesheet.yaml"
    {
        "spritesheet"
    } else if file_name.ends_with(".tileset.yml")
        || file_name.ends_with(".tileset.yaml")
        || normalized_path.contains("/spritesheets/")
            && normalized_path.contains("/tilesets/")
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "tileset"
    } else if file_name.ends_with(".tile-ruleset.yml")
        || file_name.ends_with(".tile-ruleset.yaml")
        || normalized_path.contains("/spritesheets/")
            && normalized_path.contains("/rulesets/")
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "tileset"
    } else if file_name.ends_with(".tilemap.yml") || file_name.ends_with(".tilemap.yaml") {
        "tilemap"
    } else if file_name.ends_with(".sprite.yml")
        || file_name.ends_with(".sprite.yaml")
        || file_name.ends_with(".atlas.yml")
        || file_name.ends_with(".atlas.yaml")
        || normalized_path.contains("/spritesheets/")
            && normalized_path.contains("/animations/")
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "spritesheet"
    } else if file_name.ends_with(".tileset.yml")
        || file_name.ends_with(".tileset.yaml")
        || file_name.ends_with(".tile-ruleset.yml")
        || file_name.ends_with(".tile-ruleset.yaml")
    {
        "tileset"
    } else if file_name.ends_with(".tilemap.yml") || file_name.ends_with(".tilemap.yaml") {
        "tilemap"
    } else if file_name.ends_with(".sprite.yml")
        || file_name.ends_with(".sprite.yaml")
        || file_name.ends_with(".atlas.yml")
        || file_name.ends_with(".atlas.yaml")
    {
        "spritesheet"
    } else if file_name.ends_with(".particle.yml") || file_name.ends_with(".particle.yaml") {
        "particle"
    } else if file_name.ends_with(".audio.yml") || file_name.ends_with(".audio.yaml") {
        "audio"
    } else if file_name.ends_with(".material.yml") || file_name.ends_with(".material.yaml") {
        "material"
    } else if file_name.ends_with(".ui.yml")
        || file_name.ends_with(".ui.yaml")
        || (normalized_path.starts_with("ui/") || normalized_path.contains("/ui/"))
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "ui"
    } else if file_name.ends_with(".input.yml") || file_name.ends_with(".input.yaml") {
        "input"
    } else if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp") {
        "rawImage"
    } else if matches!(extension.as_str(), "wav" | "ogg" | "mp3" | "flac") {
        "rawAudio"
    } else if matches!(extension.as_str(), "ttf" | "otf" | "woff" | "woff2") {
        "rawFont"
    } else if matches!(extension.as_str(), "yml" | "yaml") {
        "yaml"
    } else {
        "unknown"
    }
    .to_owned()
}
