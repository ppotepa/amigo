use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;

use super::signature::{ExtractedSignature, extract_signature};
use crate::model::{DependencyEntry, FileEntry, SymbolEntry};

pub fn scan_symbols(
    root: &Path,
    files: &[FileEntry],
    level: u8,
    diagnostics: &super::ScanDiagnostics,
) -> Result<Vec<SymbolEntry>> {
    let rust_patterns = RustPatterns::new()?;
    let ts_patterns = TsPatterns::new()?;
    let mut symbols = Vec::new();
    let total = files.iter().filter(|file| is_symbol_language(&file.language, level)).count();
    let mut scanned = 0usize;

    for file in files {
        if !is_symbol_language(&file.language, level) {
            continue;
        }

        scanned += 1;
        let started = std::time::Instant::now();
        if diagnostics.progress {
            eprintln!(
                "[codemap:refresh] scan_symbols {}/{} {} lang={} lines={} size={}",
                scanned,
                total,
                file.path.display(),
                file.language,
                file.lines,
                file.size
            );
        }
        let before = symbols.len();

        match file.language.as_str() {
            "rs" => symbols.extend(scan_rust(root, file, level, &rust_patterns)?),
            "ts" | "tsx" => symbols.extend(scan_ts(root, file, level, &ts_patterns)?),
            "css" if level >= 2 => symbols.extend(scan_css(root, file)?),
            "yaml" | "yml" => symbols.extend(super::yaml_mod::scan_yaml_mod_symbols(root, file)?),
            "rhai" => symbols.extend(super::rhai::scan_rhai_symbols(root, file)?),
            _ => {}
        }

        let elapsed = started.elapsed();
        let added = symbols.len().saturating_sub(before);
        if diagnostics.progress
            || elapsed.as_millis() >= u128::from(diagnostics.slow_file_threshold_ms)
        {
            eprintln!(
                "[codemap:refresh] done scan_symbols {}/{} {} in {:?} symbols+{}",
                scanned,
                total,
                file.path.display(),
                elapsed,
                added
            );
        }
    }

    Ok(symbols)
}

fn is_symbol_language(language: &str, level: u8) -> bool {
    matches!(language, "rs" | "ts" | "tsx")
        || (level >= 2 && matches!(language, "css" | "yaml" | "yml" | "rhai"))
}

pub fn scan_dependencies(
    root: &Path,
    files: &[FileEntry],
    file_ids: &BTreeMap<PathBuf, String>,
) -> Result<Vec<DependencyEntry>> {
    let mut deps = Vec::new();
    for file in files {
        match file.language.as_str() {
            "ts" | "tsx" => deps.extend(scan_ts_imports(root, file, file_ids)?),
            "rs" => deps.extend(scan_rust_mods(root, file, file_ids)?),
            _ => {}
        }
    }
    deps.sort_by(|a, b| (&a.from, &a.to, &a.kind).cmp(&(&b.from, &b.to, &b.kind)));
    deps.dedup();
    Ok(deps)
}

pub fn scan_ai_relations(
    root: &Path,
    files: &[FileEntry],
    file_ids: &BTreeMap<PathBuf, String>,
) -> Result<Vec<DependencyEntry>> {
    let mut deps = scan_test_candidate_relations(root, files, file_ids)?;
    deps.sort_by(|a, b| (&a.from, &a.to, &a.kind).cmp(&(&b.from, &b.to, &b.kind)));
    deps.dedup();
    Ok(deps)
}

fn scan_rust(
    root: &Path,
    file: &FileEntry,
    level: u8,
    patterns: &RustPatterns,
) -> Result<Vec<SymbolEntry>> {
    let text = fs::read_to_string(root.join(&file.path))?;
    let lines = text.lines().collect::<Vec<_>>();
    let mut symbols = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        if let Some(symbol) = scan_rust_field(file, level, patterns, &lines, line_index, trimmed) {
            symbols.push(symbol);
            continue;
        }
        for regex in &patterns.items {
            if let Some(caps) = regex.captures(trimmed) {
                let visibility = rust_visibility_from_caps(&caps);
                if level == 1 && !is_rust_public_visibility(&visibility) {
                    continue;
                }
                let kind = caps["kind"].to_string();
                let owner = if kind == "impl" {
                    None
                } else {
                    rust_owner_at(&lines, line_index)
                };
                let line_number = line_index + 1;
                let extracted = if level == 1 {
                    one_line_signature(trimmed, line_number, 80)
                } else {
                    extract_signature(&lines, line_index, &file.language)
                };
                symbols.push(build_symbol(
                    file,
                    caps["name"].to_string(),
                    kind,
                    line_number,
                    visibility,
                    owner,
                    extracted,
                    Vec::new(),
                ));
                break;
            }
        }
    }
    Ok(symbols)
}

fn scan_ts(
    root: &Path,
    file: &FileEntry,
    level: u8,
    patterns: &TsPatterns,
) -> Result<Vec<SymbolEntry>> {
    let text = fs::read_to_string(root.join(&file.path))?;
    let lines = text.lines().collect::<Vec<_>>();
    let mut symbols = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        if let Some(symbol) = scan_ts_field(file, level, patterns, &lines, line_index, trimmed) {
            symbols.push(symbol);
            continue;
        }
        for regex in &patterns.items {
            if let Some(caps) = regex.captures(trimmed) {
                let visibility = if caps.name("export").is_some() {
                    "export"
                } else {
                    "local"
                };
                if level == 1 && visibility != "export" {
                    continue;
                }
                let name = caps["name"].to_string();
                let mut kind = caps["kind"].to_string();
                if kind == "const" || kind == "function" {
                    kind = classify_ts_name(&name, &file.language);
                }
                let line_number = line_index + 1;
                let extracted = if level == 1 {
                    one_line_signature(trimmed, line_number, 80)
                } else {
                    extract_signature(&lines, line_index, &file.language)
                };
                let owner = ts_owner_at(&lines, line_index);
                symbols.push(build_symbol(
                    file,
                    name,
                    kind,
                    line_number,
                    visibility.to_string(),
                    owner,
                    extracted,
                    Vec::new(),
                ));
                break;
            }
        }
    }
    Ok(symbols)
}

fn scan_rust_field(
    file: &FileEntry,
    level: u8,
    patterns: &RustPatterns,
    lines: &[&str],
    line_index: usize,
    trimmed: &str,
) -> Option<SymbolEntry> {
    let caps = patterns.field.captures(trimmed)?;
    let owner = rust_struct_owner_at(lines, line_index)?;
    let visibility = rust_visibility_from_caps(&caps);
    if level == 1 && !is_rust_public_visibility(&visibility) {
        return None;
    }
    let line_number = line_index + 1;
    Some(build_symbol(
        file,
        caps["name"].to_string(),
        "field".to_string(),
        line_number,
        visibility,
        Some(owner),
        one_line_signature(trimmed, line_number, 75),
        Vec::new(),
    ))
}

fn scan_ts_field(
    file: &FileEntry,
    level: u8,
    patterns: &TsPatterns,
    lines: &[&str],
    line_index: usize,
    trimmed: &str,
) -> Option<SymbolEntry> {
    let caps = patterns.field.captures(trimmed)?;
    let owner = ts_object_owner_at(lines, line_index)?;
    if level == 1 && !owner.starts_with("interface ") {
        return None;
    }
    let line_number = line_index + 1;
    Some(build_symbol(
        file,
        caps["name"].to_string(),
        "field".to_string(),
        line_number,
        "local".to_string(),
        Some(owner),
        one_line_signature(trimmed, line_number, 75),
        Vec::new(),
    ))
}

fn scan_css(root: &Path, file: &FileEntry) -> Result<Vec<SymbolEntry>> {
    let text = fs::read_to_string(root.join(&file.path))?;
    let mut symbols = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        let Some((selector_text, _)) = line.split_once('{') else {
            continue;
        };
        let selector_text = selector_text.trim();
        if selector_text.is_empty() || selector_text.starts_with('@') {
            continue;
        }
        for selector in selector_text.split(',') {
            let selector = selector.trim();
            if selector.is_empty() {
                continue;
            }
            symbols.push(build_symbol(
                file,
                selector.to_string(),
                "css-selector".to_string(),
                line_index + 1,
                "css".to_string(),
                None,
                one_line_signature(selector, line_index + 1, 55),
                Vec::new(),
            ));
        }
    }
    Ok(symbols)
}

fn build_symbol(
    file: &FileEntry,
    name: String,
    kind: String,
    line: usize,
    visibility: String,
    owner: Option<String>,
    extracted: ExtractedSignature,
    mut tags: Vec<String>,
) -> SymbolEntry {
    tags.extend(file.tags.iter().cloned());
    tags.push(format!("kind:{kind}"));
    tags.push(format!("visibility:{visibility}"));
    tags.push(format!("lang:{}", file.language));
    if let Some(owner) = &owner {
        tags.push(format!("owner:{}", owner.replace(' ', ":")));
    }
    tags.sort();
    tags.dedup();

    SymbolEntry {
        name,
        kind,
        file_id: file.id.clone(),
        line,
        line_end: extracted.line_end,
        line_count: extracted.line_end.saturating_sub(line).saturating_add(1),
        signature: extracted.signature,
        params: extracted.params,
        return_type: extracted.return_type,
        generics: extracted.generics,
        visibility,
        owner,
        tags,
        confidence: extracted.confidence,
    }
}

fn one_line_signature(signature: &str, line_end: usize, confidence: u8) -> ExtractedSignature {
    ExtractedSignature {
        signature: signature.to_string(),
        params: Vec::new(),
        return_type: None,
        generics: Vec::new(),
        line_end,
        confidence,
    }
}

fn rust_owner_at(lines: &[&str], line_index: usize) -> Option<String> {
    let impl_re = rust_impl_re()?;
    let type_boundary_re = rust_type_boundary_re()?;
    for index in (0..line_index).rev().take(120) {
        let line = lines[index].trim_start();
        if let Some(caps) = impl_re.captures(line) {
            return Some(format!("impl {}", &caps["name"]));
        }
        if type_boundary_re.is_match(line) {
            break;
        }
    }
    None
}

fn rust_struct_owner_at(lines: &[&str], line_index: usize) -> Option<String> {
    let owner_re = rust_struct_owner_re()?;
    let stop_re = rust_struct_owner_stop_re()?;
    for index in (0..line_index).rev().take(160) {
        let line = lines[index].trim_start();
        if stop_re.is_match(line) {
            return None;
        }
        if let Some(caps) = owner_re.captures(line) {
            return Some(format!("{} {}", &caps["kind"], &caps["name"]));
        }
    }
    None
}

fn ts_owner_at(lines: &[&str], line_index: usize) -> Option<String> {
    let owner_re = ts_owner_re()?;
    for index in (0..line_index).rev().take(120) {
        let line = lines[index].trim_start();
        if let Some(caps) = owner_re.captures(line) {
            return Some(format!("{} {}", &caps["kind"], &caps["name"]));
        }
    }
    None
}

fn ts_object_owner_at(lines: &[&str], line_index: usize) -> Option<String> {
    let owner_re = ts_object_owner_re()?;
    for index in (0..line_index).rev().take(160) {
        let line = lines[index].trim_start();
        if line.starts_with("function ")
            || line.starts_with("export function ")
            || line.starts_with("class ")
            || line.starts_with("export class ")
        {
            return None;
        }
        if let Some(caps) = owner_re.captures(line) {
            return Some(format!("{} {}", &caps["kind"], &caps["name"]));
        }
    }
    None
}

fn scan_ts_imports(
    root: &Path,
    file: &FileEntry,
    file_ids: &BTreeMap<PathBuf, String>,
) -> Result<Vec<DependencyEntry>> {
    let text = fs::read_to_string(root.join(&file.path))?;
    let import_re = Regex::new(
        r#"from\s+["'](?P<path>[^"']+)["']|import\s*\(\s*["'](?P<dynamic>[^"']+)["']\s*\)"#,
    )?;
    let mut deps = Vec::new();
    for caps in import_re.captures_iter(&text) {
        let import_path = caps
            .name("path")
            .or_else(|| caps.name("dynamic"))
            .map(|m| m.as_str())
            .unwrap_or_default();
        if !import_path.starts_with('.') {
            continue;
        }
        if let Some(target) = resolve_ts_import(&file.path, import_path, file_ids) {
            deps.push(DependencyEntry {
                from: file.id.clone(),
                to: target,
                kind: "imports".to_string(),
            });
        }
    }
    Ok(deps)
}

fn scan_rust_mods(
    root: &Path,
    file: &FileEntry,
    file_ids: &BTreeMap<PathBuf, String>,
) -> Result<Vec<DependencyEntry>> {
    let text = fs::read_to_string(root.join(&file.path))?;
    let mod_re = Regex::new(r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*;")?;
    let mut deps = Vec::new();
    for caps in mod_re.captures_iter(&text) {
        let name = &caps["name"];
        if let Some(target) = resolve_rust_mod(&file.path, name, file_ids) {
            deps.push(DependencyEntry {
                from: file.id.clone(),
                to: target,
                kind: "declares".to_string(),
            });
        }
    }
    Ok(deps)
}

fn scan_test_candidate_relations(
    root: &Path,
    files: &[FileEntry],
    file_ids: &BTreeMap<PathBuf, String>,
) -> Result<Vec<DependencyEntry>> {
    let mut deps = Vec::new();
    for file in files {
        let path = slash_path(&file.path);
        if file.language == "rs" {
            let text = fs::read_to_string(root.join(&file.path))?;
            if text.contains("#[cfg(test)]") || text.contains("#[test]") {
                deps.push(DependencyEntry {
                    from: file.id.clone(),
                    to: file.id.clone(),
                    kind: "test-candidate:in-file".to_string(),
                });
            }
            continue;
        }
        if !(path.ends_with(".test.ts") || path.ends_with(".test.tsx")) {
            continue;
        }
        for source in test_source_candidates(&file.path) {
            if let Some(source_id) = file_ids.get(&source) {
                deps.push(DependencyEntry {
                    from: source_id.clone(),
                    to: file.id.clone(),
                    kind: "test-candidate".to_string(),
                });
            }
        }
    }
    Ok(deps)
}

fn test_source_candidates(test_path: &Path) -> Vec<PathBuf> {
    let path = slash_path(test_path);
    let source_paths = [
        path.strip_suffix(".test.ts")
            .map(|base| format!("{base}.ts")),
        path.strip_suffix(".test.ts")
            .map(|base| format!("{base}.tsx")),
        path.strip_suffix(".test.tsx")
            .map(|base| format!("{base}.ts")),
        path.strip_suffix(".test.tsx")
            .map(|base| format!("{base}.tsx")),
    ];
    source_paths
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect()
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn resolve_ts_import(
    from: &Path,
    import_path: &str,
    file_ids: &BTreeMap<PathBuf, String>,
) -> Option<String> {
    let parent = from.parent().unwrap_or_else(|| Path::new(""));
    let base = normalize(parent.join(import_path));
    let candidates = [
        base.clone(),
        base.with_extension("ts"),
        base.with_extension("tsx"),
        base.with_extension("js"),
        base.with_extension("jsx"),
        base.join("index.ts"),
        base.join("index.tsx"),
    ];
    candidates
        .iter()
        .find_map(|candidate| file_ids.get(candidate).cloned())
}

fn resolve_rust_mod(
    from: &Path,
    name: &str,
    file_ids: &BTreeMap<PathBuf, String>,
) -> Option<String> {
    let parent = from.parent().unwrap_or_else(|| Path::new(""));
    let candidates = [
        parent.join(format!("{name}.rs")),
        parent.join(name).join("mod.rs"),
    ];
    candidates
        .iter()
        .map(|candidate| normalize(candidate.clone()))
        .find_map(|candidate| file_ids.get(&candidate).cloned())
}

fn normalize(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn classify_ts_name(name: &str, language: &str) -> String {
    if name.starts_with("use") && name.chars().nth(3).is_some_and(char::is_uppercase) {
        return "hook".to_string();
    }
    if language == "tsx" && name.chars().next().is_some_and(char::is_uppercase) {
        return "component".to_string();
    }
    "fn".to_string()
}

struct RustPatterns {
    items: Vec<Regex>,
    field: Regex,
}

impl RustPatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            items: vec![
                Regex::new(
                    r"^(?P<vis>pub(?:\s*\([^)]*\))?\s+)?(?:async\s+|unsafe\s+|const\s+)*(?P<kind>fn)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)",
                )?,
                Regex::new(
                    r"^(?P<vis>pub(?:\s*\([^)]*\))?\s+)?(?P<kind>struct|enum|trait|mod|type|const)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)",
                )?,
                Regex::new(
                    r"^(?P<vis>pub(?:\s*\([^)]*\))?\s+)?(?P<kind>impl)\s+(?:<[^>]+>\s+)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)",
                )?,
                Regex::new(
                    r"^(?P<vis>pub(?:\s*\([^)]*\))?\s+)?(?P<kind>macro_rules!)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)",
                )?,
            ],
            field: Regex::new(
                r"^(?P<vis>pub(?:\s*\([^)]*\))?\s+)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*[^,]+,?\s*$",
            )?,
        })
    }
}

fn rust_visibility_from_caps(caps: &regex::Captures<'_>) -> String {
    caps.name("vis")
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "local".to_string())
}

fn is_rust_public_visibility(visibility: &str) -> bool {
    visibility == "pub" || visibility.starts_with("pub(")
}

fn rust_impl_re() -> Option<&'static Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    Some(RE.get_or_init(|| {
        Regex::new(r"^\s*impl(?:\s*<[^>]+>)?\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid impl regex")
    }))
}

fn rust_type_boundary_re() -> Option<&'static Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    Some(RE.get_or_init(|| {
        Regex::new(r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:struct|enum|trait)\s+[A-Za-z_][A-Za-z0-9_]*")
            .expect("valid type boundary regex")
    }))
}

fn rust_struct_owner_re() -> Option<&'static Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    Some(RE.get_or_init(|| {
        Regex::new(r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?P<kind>struct|enum)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid struct owner regex")
    }))
}

fn rust_struct_owner_stop_re() -> Option<&'static Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    Some(RE.get_or_init(|| {
        Regex::new(
            r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:(?:async|unsafe|const)\s+)*fn\s+|^\s*impl\s+|^\s*(?:pub(?:\s*\([^)]*\))?\s+)?trait\s+",
        )
        .expect("valid struct owner stop regex")
    }))
}

fn ts_owner_re() -> Option<&'static Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    Some(RE.get_or_init(|| {
        Regex::new(r"^\s*(?:export\s+)?(?P<kind>class|interface|type)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid ts owner regex")
    }))
}

fn ts_object_owner_re() -> Option<&'static Regex> {
    static RE: OnceLock<Regex> = OnceLock::new();
    Some(RE.get_or_init(|| {
        Regex::new(r"^\s*(?:export\s+)?(?P<kind>interface|type)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid ts object owner regex")
    }))
}

struct TsPatterns {
    items: Vec<Regex>,
    field: Regex,
}

impl TsPatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            items: vec![
                Regex::new(
                    r"^(?P<export>export\s+)?(?P<kind>function)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)",
                )?,
                Regex::new(
                    r"^(?P<export>export\s+)?(?P<kind>const|let|var)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*=",
                )?,
                Regex::new(
                    r"^(?P<export>export\s+)?(?P<kind>type|interface|enum|class)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)",
                )?,
            ],
            field: Regex::new(
                r"^(?:readonly\s+)?(?P<name>[A-Za-z_$][A-Za-z0-9_$]*)\??\s*:\s*.+[,;]?\s*$",
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::model::FileEntry;

    use super::{RustPatterns, TsPatterns, classify_ts_name, scan_rust, scan_ts};

    #[test]
    fn scans_rust_pub_crate_items_at_level_one() {
        let root = temp_root("rust-pub-crate-items");
        std::fs::write(
            root.join("console.rs"),
            "pub(crate) fn register_builtin_console_commands() {}\n\
             pub(crate) struct ResolvedDevConsoleOverlayExtractor;\n\
             pub(super) async fn build_dev_console_overlay() {}\n",
        )
        .expect("write rs");
        let file = FileEntry {
            id: "f1".to_string(),
            path: PathBuf::from("console.rs"),
            language: "rs".to_string(),
            ..Default::default()
        };

        let symbols = scan_rust(Path::new(&root), &file, 1, &RustPatterns::new().unwrap()).unwrap();

        assert!(symbols.iter().any(|symbol| {
            symbol.name == "register_builtin_console_commands"
                && symbol.kind == "fn"
                && symbol.visibility == "pub(crate)"
        }));
        assert!(symbols.iter().any(|symbol| {
            symbol.name == "ResolvedDevConsoleOverlayExtractor"
                && symbol.kind == "struct"
                && symbol.visibility == "pub(crate)"
        }));
        assert!(symbols.iter().any(|symbol| {
            symbol.name == "build_dev_console_overlay"
                && symbol.kind == "fn"
                && symbol.visibility == "pub(super)"
        }));
    }

    #[test]
    fn scans_rust_pub_crate_struct_fields_with_owner() {
        let root = temp_root("rust-pub-crate-fields");
        std::fs::write(
            root.join("overlay.rs"),
            "pub(crate) struct ResolvedDevConsoleOverlayExtractor {\n    pub(crate) enabled: bool,\n}\n",
        )
        .expect("write rs");
        let file = FileEntry {
            id: "f1".to_string(),
            path: PathBuf::from("overlay.rs"),
            language: "rs".to_string(),
            ..Default::default()
        };

        let symbols = scan_rust(Path::new(&root), &file, 1, &RustPatterns::new().unwrap()).unwrap();

        assert!(symbols.iter().any(|symbol| {
            symbol.name == "enabled"
                && symbol.kind == "field"
                && symbol.visibility == "pub(crate)"
                && symbol.owner.as_deref() == Some("struct ResolvedDevConsoleOverlayExtractor")
        }));
    }

    #[test]
    fn scans_rust_struct_fields() {
        let root = temp_root("rust-fields");
        std::fs::write(
            root.join("dto.rs"),
            "pub struct EditorUiNodeDto {\n    pub action_target: Option<String>,\n}\n",
        )
        .expect("write rs");
        let file = FileEntry {
            id: "f1".to_string(),
            path: PathBuf::from("dto.rs"),
            language: "rs".to_string(),
            ..Default::default()
        };

        let symbols = scan_rust(Path::new(&root), &file, 2, &RustPatterns::new().unwrap()).unwrap();

        assert!(symbols.iter().any(|symbol| {
            symbol.name == "action_target"
                && symbol.kind == "field"
                && symbol.owner.as_deref() == Some("struct EditorUiNodeDto")
        }));
    }

    #[test]
    fn classifies_tsx_components_and_hooks() {
        assert_eq!(classify_ts_name("StartupDialog", "tsx"), "component");
        assert_eq!(classify_ts_name("useEditorStore", "tsx"), "hook");
        assert_eq!(classify_ts_name("scanMods", "ts"), "fn");
    }

    #[test]
    fn scans_ts_interface_fields() {
        let root = temp_root("ts-fields");
        std::fs::write(
            root.join("dto.ts"),
            "export interface EditorUiNodeDto {\n  actionTarget?: string | null;\n}\n",
        )
        .expect("write ts");
        let file = FileEntry {
            id: "f1".to_string(),
            path: PathBuf::from("dto.ts"),
            language: "ts".to_string(),
            ..Default::default()
        };

        let symbols = scan_ts(Path::new(&root), &file, 2, &TsPatterns::new().unwrap()).unwrap();

        assert!(symbols.iter().any(|symbol| {
            symbol.name == "actionTarget"
                && symbol.kind == "field"
                && symbol.owner.as_deref() == Some("interface EditorUiNodeDto")
        }));
    }


    fn temp_root(name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should advance")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("amigo-codemap-symbols-{name}-{unique}"));
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }
}
