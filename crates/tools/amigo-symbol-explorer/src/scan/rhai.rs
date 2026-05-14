use std::fs;
use std::path::Path;

use anyhow::Result;
use regex::Regex;

use crate::model::{FileEntry, SymbolEntry};

/// @codemap(P1): amigo-rhai-script-scanner
/// Amigo-specific Rhai scanner for script functions and world/scene API usage.
pub fn scan_rhai_symbols(root: &Path, file: &FileEntry) -> Result<Vec<SymbolEntry>> {
    let text = fs::read_to_string(root.join(&file.path))?;
    let fn_re = Regex::new(r"^\s*fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\(")?;
    let api_re = Regex::new(
        r"(?P<name>(?:world|scene|entity|particles|layered_image2d|render|assets)\.[A-Za-z0-9_\.]+)\s*\(",
    )?;

    let mut symbols = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        if let Some(caps) = fn_re.captures(line) {
            symbols.push(symbol(
                file,
                caps["name"].to_string(),
                "fn",
                line_index + 1,
                line,
                80,
            ));
        }

        for caps in api_re.captures_iter(line) {
            symbols.push(symbol(
                file,
                caps["name"].to_string(),
                "rhai-api-call",
                line_index + 1,
                line,
                70,
            ));
        }
    }

    Ok(symbols)
}

fn symbol(
    file: &FileEntry,
    name: String,
    kind: &str,
    line: usize,
    source_line: &str,
    confidence: u8,
) -> SymbolEntry {
    let mut tags = file.tags.clone();
    tags.push(format!("kind:{kind}"));
    tags.push("lang:rhai".to_string());
    tags.sort();
    tags.dedup();

    SymbolEntry {
        name,
        kind: kind.to_string(),
        file_id: file.id.clone(),
        line,
        line_end: line,
        line_count: 1,
        signature: trim_signature(source_line.trim(), line),
        params: Vec::new(),
        return_type: None,
        generics: Vec::new(),
        visibility: "rhai".to_string(),
        owner: None,
        tags,
        confidence,
    }
}

fn trim_signature(line: &str, line_number: usize) -> String {
    let trimmed = if line.len() > 90 {
        format!("{}...", &line[..90])
    } else {
        line.to_string()
    };
    format!("{line_number}: {trimmed}")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::model::FileEntry;

    use super::scan_rhai_symbols;

    #[test]
    fn scans_rhai_functions_and_world_api_calls() {
        let root = temp_root("rhai-api");
        std::fs::write(
            root.join("script.rhai"),
            "fn on_start() {\n  world.layered_image2d.set_layer_opacity(\"bg\", \"club\", 0.5);\n}\n",
        )
        .expect("write");

        let file = FileEntry {
            id: "f1".to_string(),
            path: PathBuf::from("script.rhai"),
            language: "rhai".to_string(),
            lines: 3,
            hash: "hash".to_string(),
            size: 10,
            tags: vec!["lang:rhai".to_string()],
        };

        let symbols = scan_rhai_symbols(&root, &file).expect("scan");
        assert!(
            symbols
                .iter()
                .any(|symbol| symbol.kind == "fn" && symbol.name == "on_start")
        );
        assert!(symbols.iter().any(|symbol| {
            symbol.kind == "rhai-api-call"
                && symbol.name == "world.layered_image2d.set_layer_opacity"
        }));
    }

    fn temp_root(name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should advance")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("amigo-rhai-{name}-{unique}"));
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }
}
