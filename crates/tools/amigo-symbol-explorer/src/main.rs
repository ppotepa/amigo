use std::path::{Path, PathBuf};

use amigo_symbol_explorer::scan::{ScanDiagnostics, SymbolExplorerScanOptions, scan_project};
use anyhow::Result;

fn main() -> Result<()> {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let root = std::env::current_dir()?.join(root);

    let map = scan_project(&SymbolExplorerScanOptions {
        root: root.clone(),
        level: 2,
        ai: false,
        diagnostics: ScanDiagnostics::default(),
    })?;

    let plugin = plugin_label(&root);
    println!("plugin: {plugin}");
    println!("root: {}", slash_path(&root));
    println!("files: {}", map.files.len());
    println!("symbols: {}", map.symbols.len());
    println!("packages: {}", map.packages.len());

    println!("file ownership:");
    for file in map.files.iter().take(40) {
        println!(
            "  {}\t{}\t{}",
            plugin,
            slash_path(&file.path),
            file.tags.join(",")
        );
    }

    println!("symbols per plugin:");
    for symbol in map.symbols.iter().take(80) {
        println!(
            "  {}\t{}\t{}\t{}:{}",
            plugin, symbol.kind, symbol.name, symbol.file_id, symbol.line
        );
    }

    Ok(())
}

fn plugin_label(root: &Path) -> String {
    let parts = root
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if let Some(index) = parts.iter().position(|part| part == "plugins") {
        if let (Some(family), Some(plugin)) = (parts.get(index + 1), parts.get(index + 2)) {
            return format!("{family}/{plugin}");
        }
    }

    root.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_owned())
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
