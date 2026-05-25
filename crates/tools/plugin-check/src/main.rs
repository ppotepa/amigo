use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use amigo_codemap_api::{validate_codemap_graph, CodeMapNodeId};
use amigo_plugin_index::{build_codemap_graph_from_index, validate_plugin_index, PluginIndex};
use amigo_plugin_loader::load_plugin_manifests_from_plugins_dir;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "validate") {
        let roots = parse_validate_roots(&args[1..]);
        if let Err(errors) = validate_plugin_tree(&roots) {
            eprintln!("plugin tree validation failed:");
            for error in errors {
                eprintln!("- {error}");
            }
            std::process::exit(1);
        }
        println!("plugin tree validation passed");
        return;
    }

    let first_tail = args.get(1).cloned();
    let mut args = args.into_iter();
    let first = args.next().unwrap_or_else(|| "summary".to_owned());
    let known_command = matches!(
        first.as_str(),
        "summary" | "check" | "plugins" | "targets" | "diagnostics" | "graph"
    ) && !(first == "plugins" && first_tail.is_some());
    let command = if known_command {
        first.clone()
    } else {
        "check".to_owned()
    };
    let roots: Vec<PathBuf> = if known_command {
        let roots: Vec<PathBuf> = args.map(PathBuf::from).collect();
        if roots.is_empty() {
            vec![PathBuf::from("plugins")]
        } else {
            roots
        }
    } else {
        std::iter::once(PathBuf::from(first))
            .chain(args.map(PathBuf::from))
            .collect()
    };

    if let Err(errors) = validate_plugin_tree(&roots) {
        eprintln!("plugin tree validation failed:");
        for error in errors {
            eprintln!("- {error}");
        }
        std::process::exit(1);
    }

    let mut manifests = Vec::new();
    for root in &roots {
        match load_plugin_manifests_from_plugins_dir(root) {
            Ok(root_manifests) => manifests.extend(root_manifests),
            Err(errors) => {
                eprintln!("plugin load failed: {errors:#?}");
                std::process::exit(1);
            }
        }
    }

    let index = PluginIndex::from_manifests(manifests);

    if let Err(errors) = validate_plugin_index(&index) {
        eprintln!("plugin index validation failed: {errors:#?}");
        std::process::exit(1);
    }

    let graph = build_codemap_graph_from_index(&index);

    if let Err(errors) = validate_codemap_graph(&graph) {
        eprintln!("codemap graph validation failed: {errors:#?}");
        std::process::exit(1);
    }

    match command.as_str() {
        "summary" | "check" => print_summary(&index, graph.nodes.len(), graph.edges.len()),
        "plugins" => print_plugins(&index),
        "targets" => print_targets(&graph),
        "diagnostics" => print_diagnostics(&graph),
        "graph" => print_graph(&graph),
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: amigo-plugin-check [summary|plugins|targets|diagnostics|graph] [plugins_dir]");
            std::process::exit(2);
        }
    }
}

fn parse_validate_roots(args: &[String]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--workspace" => {}
            "--plugins" => {
                if let Some(path) = iter.next() {
                    roots.push(PathBuf::from(path));
                }
            }
            other => roots.push(PathBuf::from(other)),
        }
    }
    if roots.is_empty() {
        roots.push(PathBuf::from("plugins"));
    }
    roots
}

fn validate_plugin_tree(roots: &[PathBuf]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let plugin_dirs: Vec<PathBuf> = roots
        .iter()
        .flat_map(|root| collect_plugin_dirs(root, &mut errors))
        .collect();
    let mut ids = BTreeSet::new();

    for plugin_dir in plugin_dirs {
        validate_plugin_dir(&plugin_dir, &mut ids, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_plugin_dirs(root: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    if root.join("plugin.toml").exists() {
        return vec![root.to_path_buf()];
    }

    let mut plugins = Vec::new();
    let Ok(families) = fs::read_dir(root) else {
        errors.push(format!("{} is not readable", root.display()));
        return plugins;
    };

    for family in families.flatten() {
        let family_path = family.path();
        if !family_path.is_dir() {
            continue;
        }

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

    require_file(plugin_dir, "plugin.toml", errors);
    require_file(plugin_dir, "Cargo.toml", errors);
    require_file(plugin_dir, "README.md", errors);
    require_file(plugin_dir, "tests/waterfall_tests.rs", errors);
    require_file(plugin_dir, "src/plugin.rs", errors);

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
        errors.push(format!(
            "{} must use src/render_wgpu, not src/render-wgpu",
            plugin_dir.display()
        ));
    }

    if manifest_path.exists() {
        validate_manifest_identity(plugin_dir, &manifest_path, ids, errors);
        validate_manifest_referenced_files(plugin_dir, &manifest_path, errors);
    }

    if cargo_path.exists() {
        validate_cargo_package_name(plugin_dir, &cargo_path, errors);
    }

    validate_forbidden_patterns(plugin_dir, errors);
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

fn validate_manifest_identity(
    plugin_dir: &Path,
    manifest_path: &Path,
    ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let Ok(content) = fs::read_to_string(manifest_path) else {
        errors.push(format!("{} is not readable", manifest_path.display()));
        return;
    };

    let id = manifest_scalar(&content, "id");
    let family = manifest_scalar(&content, "family");
    let actual_family = plugin_dir
        .parent()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("");

    match family {
        Some(family) if family == actual_family => {}
        Some(family) => errors.push(format!(
            "{} family `{family}` does not match plugins/{actual_family}",
            manifest_path.display()
        )),
        None => errors.push(format!("{} missing family", manifest_path.display())),
    }

    match id {
        Some(id) if ids.insert(id.to_owned()) => {}
        Some(id) => errors.push(format!("duplicate plugin id `{id}`")),
        None => errors.push(format!("{} missing id", manifest_path.display())),
    }
}

fn validate_cargo_package_name(plugin_dir: &Path, cargo_path: &Path, errors: &mut Vec<String>) {
    let Ok(content) = fs::read_to_string(cargo_path) else {
        errors.push(format!("{} is not readable", cargo_path.display()));
        return;
    };

    if manifest_scalar(&content, "name").is_none() {
        errors.push(format!("{} missing package name", plugin_dir.display()));
    }
}

fn validate_manifest_referenced_files(
    plugin_dir: &Path,
    manifest_path: &Path,
    errors: &mut Vec<String>,
) {
    let Ok(content) = fs::read_to_string(manifest_path) else {
        errors.push(format!("{} is not readable", manifest_path.display()));
        return;
    };

    for section in ["tests"] {
        for (key, relative) in manifest_section_scalars(&content, section) {
            if relative.trim().is_empty() {
                errors.push(format!(
                    "{} [{section}].{key} references an empty path",
                    manifest_path.display()
                ));
                continue;
            }

            let relative_path = Path::new(relative);
            if relative_path.is_absolute()
                || relative_path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
            {
                errors.push(format!(
                    "{} [{section}].{key} must reference a plugin-relative file, got `{relative}`",
                    manifest_path.display()
                ));
                continue;
            }

            let referenced = plugin_dir.join(relative_path);
            if !referenced.is_file() {
                errors.push(format!(
                    "{} [{section}].{key} references missing file `{relative}`",
                    manifest_path.display()
                ));
            }
        }
    }
}

fn manifest_section_scalars<'a>(content: &'a str, section: &str) -> Vec<(&'a str, &'a str)> {
    let section_header = format!("[{section}]");
    let mut in_section = false;
    let mut scalars = Vec::new();

    for line in content.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == section_header;
            continue;
        }

        if !in_section || line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        let Some(value) = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        scalars.push((key, value));
    }

    scalars
}

fn manifest_scalar<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    content.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix(&prefix)
            .and_then(|value| value.trim().strip_prefix('"'))
            .and_then(|value| value.strip_suffix('"'))
    })
}

fn validate_forbidden_patterns(plugin_dir: &Path, errors: &mut Vec<String>) {
    const FORBIDDEN: &[&str] = &[
        concat!("leg", "acy"),
        concat!("depre", "cated"),
        concat!("_", "v", "2"),
        concat!("re", "-", "export"),
        concat!("re", "export"),
        concat!("luma", "_fallback"),
        concat!("should", "_produce", "_scene", "_highlight"),
        concat!("direct", "_lens", "_flare"),
        concat!("guess", "_optical"),
        concat!("flare", "_strength"),
        concat!("lens", "_influence"),
    ];

    for path in files_under(plugin_dir) {
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        for forbidden in FORBIDDEN {
            if content.contains(forbidden) {
                errors.push(format!(
                    "{} contains forbidden `{forbidden}`",
                    path.display()
                ));
            }
        }
    }
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, &mut files);
    files
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some("target") {
                continue;
            }
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn print_summary(index: &PluginIndex, nodes: usize, edges: usize) {
    println!("plugins: {}", index.len());
    println!("nodes: {nodes}");
    println!("edges: {edges}");
}

fn print_plugins(index: &PluginIndex) {
    let mut plugin_ids = index
        .manifests()
        .map(|manifest| manifest.id.0.as_str())
        .collect::<Vec<_>>();
    plugin_ids.sort_unstable();

    for plugin_id in plugin_ids {
        println!("{plugin_id}");
    }
}

fn print_targets(graph: &amigo_codemap_api::CodeMapGraph) {
    print_nodes_by_kind(graph, |id| matches!(id, CodeMapNodeId::Target(_)));
}

fn print_diagnostics(graph: &amigo_codemap_api::CodeMapGraph) {
    print_nodes_by_kind(graph, |id| {
        matches!(id, CodeMapNodeId::DiagnosticChannel(_))
    });
}

fn print_graph(graph: &amigo_codemap_api::CodeMapGraph) {
    for edge in &graph.edges {
        println!("{:?} --{:?}--> {:?}", edge.from, edge.kind, edge.to);
    }
}

fn print_nodes_by_kind(
    graph: &amigo_codemap_api::CodeMapGraph,
    predicate: impl Fn(&CodeMapNodeId) -> bool,
) {
    let mut labels = graph
        .nodes
        .values()
        .filter(|node| predicate(&node.id))
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();
    labels.sort_unstable();
    labels.dedup();

    for label in labels {
        println!("{label}");
    }
}
