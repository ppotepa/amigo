use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use amigo_plugin_manifest::parse_plugin_manifest_str;

pub(crate) fn parse_validate_roots(args: &[String]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--workspace" => {}
            "--plugins" => if let Some(path) = iter.next() { roots.push(PathBuf::from(path)); },
            other => roots.push(PathBuf::from(other)),
        }
    }
    if roots.is_empty() { roots.push(PathBuf::from("plugins")); }
    roots
}

pub(crate) fn validate_plugin_tree(roots: &[PathBuf]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let plugin_dirs: Vec<PathBuf> = roots
        .iter()
        .flat_map(|root| collect_plugin_dirs(root, &mut errors))
        .collect();
    let mut ids = BTreeSet::new();
    for plugin_dir in plugin_dirs {
        validate_plugin_dir(&plugin_dir, &mut ids, &mut errors);
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn collect_plugin_dirs(root: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    if root.join("plugin.toml").exists() { return vec![root.to_path_buf()]; }
    let mut plugins = Vec::new();
    let Ok(families) = fs::read_dir(root) else {
        errors.push(format!("{} is not readable", root.display()));
        return plugins;
    };
    for family in families.flatten() {
        let family_path = family.path();
        if !family_path.is_dir() { continue; }
        let Ok(entries) = fs::read_dir(&family_path) else {
            errors.push(format!("{} is not readable", family_path.display()));
            continue;
        };
        for entry in entries.flatten() {
            let plugin_path = entry.path();
            if plugin_path.is_dir() && plugin_path.join("plugin.toml").exists() {
                plugins.push(plugin_path);
            }
        }
    }
    plugins
}

fn validate_plugin_dir(plugin_dir: &Path, ids: &mut BTreeSet<String>, errors: &mut Vec<String>) {
    let manifest_path = plugin_dir.join("plugin.toml");
    let cargo_path = plugin_dir.join("Cargo.toml");
    for file in ["plugin.toml", "Cargo.toml", "README.md", "tests/waterfall_tests.rs", "src/plugin.rs"] {
        require_file(plugin_dir, file, errors);
    }
    validate_waterfall_test(plugin_dir, errors);
    for dir in ["src/api", "src/scene", "src/runtime", "src/scripting", "src/diagnostics"] {
        require_dir(plugin_dir, dir, errors);
    }
    if !plugin_dir.join("src/render_wgpu").is_dir() && !plugin_dir.join("src/render").is_dir() {
        errors.push(format!(
            "{} missing render boundary directory: expected src/render_wgpu or src/render",
            plugin_dir.display()
        ));
    }
    if plugin_dir.join("src/render-wgpu").exists() {
        errors.push(format!("{} must use src/render_wgpu, not src/render-wgpu", plugin_dir.display()));
    }

    if manifest_path.exists() {
        validate_manifest(plugin_dir, &manifest_path, ids, errors);
    }
    if cargo_path.exists() {
        validate_cargo_package_name(plugin_dir, &cargo_path, errors);
    }
    validate_forbidden_patterns(plugin_dir, errors);
}

fn validate_manifest(
    plugin_dir: &Path,
    manifest_path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let Ok(content) = fs::read_to_string(manifest_path) else {
        errors.push(format!("{} is not readable", manifest_path.display()));
        return;
    };
    let manifest = match parse_plugin_manifest_str(&content) {
        Ok(manifest) => manifest,
        Err(error) => {
            errors.push(format!("{} does not parse: {error:?}", manifest_path.display()));
            return;
        }
    };

    let actual_family = plugin_dir
        .parent()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if manifest.family.0 != actual_family {
        errors.push(format!(
            "{} family `{}` does not match plugins/{actual_family}",
            manifest_path.display(), manifest.family.0
        ));
    }
    if !ids.insert(manifest.id.0.clone()) {
        errors.push(format!("duplicate plugin id `{}`", manifest.id.0));
    }

    let references = [
        ("docs.pipeline", manifest.docs.pipeline.as_deref()),
        ("docs.contributions", manifest.docs.contributions.as_deref()),
        ("docs.diagnostics", manifest.docs.diagnostics.as_deref()),
        ("tests.hydration", manifest.tests.hydration.as_deref()),
        ("tests.participation", manifest.tests.participation.as_deref()),
        ("tests.candidate", manifest.tests.candidate.as_deref()),
        ("tests.waterfall", manifest.tests.waterfall.as_deref()),
        ("tests.diagnostics", manifest.tests.diagnostics.as_deref()),
    ];
    for (field, relative) in references {
        if let Some(relative) = relative {
            validate_relative_reference(plugin_dir, manifest_path, field, relative, errors);
        }
    }
}

fn validate_relative_reference(
    plugin_dir: &Path,
    manifest_path: &Path,
    field: &str,
    relative: &str,
    errors: &mut Vec<String>,
) {
    if relative.trim().is_empty() {
        errors.push(format!("{} {field} references an empty path", manifest_path.display()));
        return;
    }
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| matches!(component, std::path::Component::ParentDir))
    {
        errors.push(format!(
            "{} {field} must reference a plugin-relative file, got `{relative}`",
            manifest_path.display()
        ));
        return;
    }
    if !plugin_dir.join(relative_path).is_file() {
        errors.push(format!(
            "{} {field} references missing file `{relative}`",
            manifest_path.display()
        ));
    }
}

fn validate_cargo_package_name(plugin_dir: &Path, cargo_path: &Path, errors: &mut Vec<String>) {
    let Ok(content) = fs::read_to_string(cargo_path) else {
        errors.push(format!("{} is not readable", cargo_path.display()));
        return;
    };
    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(format!("{} does not parse: {error}", cargo_path.display()));
            return;
        }
    };
    let package_name = parsed
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str);
    if package_name.is_none_or(str::is_empty) {
        errors.push(format!("{} missing [package].name", plugin_dir.display()));
    }
}

fn validate_waterfall_test(plugin_dir: &Path, errors: &mut Vec<String>) {
    let path = plugin_dir.join("tests/waterfall_tests.rs");
    let Ok(content) = fs::read_to_string(&path) else { return; };
    let compact: String = content.chars().filter(|ch| !ch.is_whitespace()).collect();
    if !content.contains("#[test]") {
        errors.push(format!("{} must contain at least one #[test]", path.display()));
    }
    if compact.contains("assert!(true)") || compact.contains("assert_eq!(true,true)") {
        errors.push(format!("{} contains a trivial placeholder assertion", path.display()));
    }
    let lowercase = content.to_ascii_lowercase();
    let semantic_markers = ["candidate", "target", "contribution", "diagnostic", "descriptor", "register", "render"];
    if !semantic_markers.iter().any(|marker| lowercase.contains(marker)) {
        errors.push(format!("{} does not exercise any plugin waterfall stage", path.display()));
    }
}

fn require_file(plugin_dir: &Path, relative: &str, errors: &mut Vec<String>) {
    if !plugin_dir.join(relative).is_file() {
        errors.push(format!("{} missing {relative}", plugin_dir.display()));
    }
}
fn require_dir(plugin_dir: &Path, relative: &str, errors: &mut Vec<String>) {
    if !plugin_dir.join(relative).is_dir() {
        errors.push(format!("{} missing {relative}", plugin_dir.display()));
    }
}

fn validate_forbidden_patterns(plugin_dir: &Path, errors: &mut Vec<String>) {
    const FORBIDDEN_IDENTIFIERS: &[&str] = &[
        "luma_fallback", "should_produce_scene_highlight", "direct_lens_flare", "guess_optical",
        "flare_strength", "lens_influence",
    ];
    for path in files_under(plugin_dir) {
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") { continue; }
        let Ok(content) = fs::read_to_string(&path) else { continue; };
        for line in content.lines().map(str::trim) {
            if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') { continue; }
            for forbidden in FORBIDDEN_IDENTIFIERS {
                if contains_identifier(line, forbidden) {
                    errors.push(format!("{} contains forbidden identifier `{forbidden}`", path.display()));
                }
            }
        }
    }
}

fn contains_identifier(line: &str, identifier: &str) -> bool {
    line.match_indices(identifier).any(|(start, _)| {
        let before = line[..start].chars().next_back();
        let end = start + identifier.len();
        let after = line[end..].chars().next();
        !before.is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
            && !after.is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
    })
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, &mut files);
    files
}
fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else { return; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some("target") { continue; }
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}
