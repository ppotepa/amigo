use std::fs;
use std::path::Path;

use anyhow::Result;
use regex::Regex;

use crate::model::{FileEntry, SymbolEntry};

/// @codemap(P0): amigo-yaml-mod-scanner
/// Amigo-specific YAML scanner for mods, scenes, UI documents, and asset descriptors.
pub fn scan_yaml_mod_symbols(root: &Path, file: &FileEntry) -> Result<Vec<SymbolEntry>> {
    let text = fs::read_to_string(root.join(&file.path))?;
    let key_value =
        Regex::new(r##"^\s*-?\s*(?P<key>[A-Za-z0-9_.:-]+)\s*:\s*['"]?(?P<value>[^'"#]+)"##)?;
    let mut symbols = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let Some(caps) = key_value.captures(line) else {
            continue;
        };

        let key = caps.name("key").map(|m| m.as_str()).unwrap_or_default();
        let value = caps
            .name("value")
            .map(|m| clean_value(m.as_str()))
            .unwrap_or_default();
        if value.is_empty() || should_skip_value(&value) {
            continue;
        }

        let Some(kind) = classify_yaml_symbol(file, key, &value) else {
            continue;
        };

        symbols.push(SymbolEntry {
            name: value,
            kind: kind.to_string(),
            file_id: file.id.clone(),
            line: line_index + 1,
            line_end: line_index + 1,
            line_count: 1,
            signature: trim_signature(line.trim(), line_index + 1),
            params: Vec::new(),
            return_type: None,
            generics: Vec::new(),
            visibility: "yaml".to_string(),
            owner: None,
            tags: yaml_tags(file, key, kind),
            confidence: 75,
        });
    }

    Ok(symbols)
}

fn classify_yaml_symbol(file: &FileEntry, key: &str, value: &str) -> Option<&'static str> {
    let path = slash_path(&file.path);
    let key = key.to_ascii_lowercase();

    if key == "kind" {
        return Some("asset-kind");
    }
    if key == "type" && path.contains("/scenes/") {
        return Some("component");
    }
    if matches!(key.as_str(), "asset" | "asset_id" | "asset_ref") {
        return Some("asset-ref");
    }
    if matches!(key.as_str(), "image" | "file" | "source") && looks_like_source_file(value) {
        return Some("source-file");
    }
    if matches!(key.as_str(), "script" | "script_file" | "rhai") {
        return Some("script-ref");
    }
    if matches!(key.as_str(), "scene" | "scene_id" | "target_scene") {
        return Some("scene-ref");
    }
    if key == "event" || key.ends_with("_event") {
        return Some("ui-event");
    }
    if matches!(key.as_str(), "id" | "name") {
        if path.contains("/scenes/") {
            return Some("scene-or-entity");
        }
        if path.contains("/ui") || path.contains("ui-document") {
            return Some("ui-node");
        }
        if path.contains("/layered-images/") {
            return Some("layered-image");
        }
        if path.contains("/fonts/") {
            return Some("font");
        }
        return Some("yaml-id");
    }

    None
}

fn yaml_tags(file: &FileEntry, key: &str, kind: &str) -> Vec<String> {
    let mut tags = file.tags.clone();
    tags.push(format!("kind:{kind}"));
    tags.push(format!("yaml-key:{key}"));

    let path = slash_path(&file.path);
    if path.contains("/scenes/") {
        tags.push("domain:yaml:scene".to_string());
    }
    if path.contains("/layered-images/") {
        tags.push("domain:yaml:layered-image".to_string());
    }
    if path.contains("/fonts/") {
        tags.push("domain:yaml:font".to_string());
    }
    if path.contains("/ui") || path.contains("ui-document") {
        tags.push("domain:yaml:ui".to_string());
    }

    tags.sort();
    tags.dedup();
    tags
}

fn looks_like_source_file(value: &str) -> bool {
    matches!(
        value
            .rsplit('.')
            .next()
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(
            "png"
                | "jpg"
                | "jpeg"
                | "webp"
                | "ttf"
                | "otf"
                | "wav"
                | "ogg"
                | "rhai"
                | "yml"
                | "yaml"
        )
    )
}

fn should_skip_value(value: &str) -> bool {
    value.len() < 2 || value.len() > 180 || value.starts_with('{') || value.starts_with('[')
}

fn clean_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches(',')
        .to_string()
}

fn trim_signature(line: &str, line_number: usize) -> String {
    let trimmed = if line.len() > 90 {
        format!("{}...", &line[..90])
    } else {
        line.to_string()
    };
    format!("{line_number}: {trimmed}")
}

fn slash_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::model::FileEntry;

    use super::scan_yaml_mod_symbols;

    #[test]
    fn scans_layered_image_source_files() {
        let root = temp_root("yaml-layered-image");
        std::fs::create_dir_all(root.join("mods/game/layered-images/neon")).expect("mkdir");
        std::fs::write(
            root.join("mods/game/layered-images/neon/layered-image.yml"),
            "kind: layered-image-2d\nid: neon\nbase:\n  image: base_albedo.png\nlayers:\n  - id: club_sign\n    image: light_001.png\n",
        )
        .expect("write");

        let file = FileEntry {
            id: "f1".to_string(),
            path: PathBuf::from("mods/game/layered-images/neon/layered-image.yml"),
            language: "yaml".to_string(),
            lines: 6,
            hash: "hash".to_string(),
            size: 10,
            tags: vec!["lang:yaml".to_string()],
        };

        let symbols = scan_yaml_mod_symbols(&root, &file).expect("scan");
        assert!(
            symbols
                .iter()
                .any(|symbol| symbol.kind == "source-file" && symbol.name == "base_albedo.png")
        );
        assert!(
            symbols
                .iter()
                .any(|symbol| symbol.kind == "source-file" && symbol.name == "light_001.png")
        );
    }

    fn temp_root(name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should advance")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("amigo-yaml-mod-{name}-{unique}"));
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }
}

