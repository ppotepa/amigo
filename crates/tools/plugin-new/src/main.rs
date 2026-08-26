use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(options) = PluginNewOptions::from_args(&args) else {
        eprintln!("usage: amigo-plugin-new --family <family> --name <plugin-name> --kind <kind> [--renderable true|false]");
        std::process::exit(2);
    };

    let source = PathBuf::from("templates/plugin");
    let target = PathBuf::from("plugins").join(&options.family).join(&options.name);
    if target.exists() {
        eprintln!("target already exists: {}", target.display());
        std::process::exit(1);
    }

    if let Err(error) = generate_plugin(&source, &target, &options) {
        let _ = fs::remove_dir_all(&target);
        eprintln!("plugin generation failed: {error}");
        std::process::exit(1);
    }

    if let Err(error) = validate_generated_plugin(&target) {
        let _ = fs::remove_dir_all(&target);
        eprintln!("generated plugin did not pass plugin-check: {error}");
        std::process::exit(1);
    }

    println!("{}", target.display());
}

fn generate_plugin(source: &Path, target: &Path, options: &PluginNewOptions) -> Result<(), String> {
    copy_dir(source, target).map_err(|error| error.to_string())?;
    rewrite_text_tree(target, options)?;
    Ok(())
}

fn rewrite_text_tree(path: &Path, options: &PluginNewOptions) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            rewrite_text_tree(&path, options)?;
            continue;
        }
        let Ok(source) = fs::read_to_string(&path) else { continue; };
        let plugin_id = format!("amigo.{}.{}", options.family, options.name);
        let rewritten = source
            .replace("amigo.family.plugin-name", &plugin_id)
            .replace("family = \"family\"", &format!("family = \"{}\"", options.family))
            .replace("kind = \"noop\"", &format!("kind = \"{}\"", options.kind))
            .replace("renderable = false", &format!("renderable = {}", options.renderable))
            .replace("amigo-plugin-template", &format!("amigo-{}-plugin", options.name))
            .replace("plugins/family/plugin-name", &format!("plugins/{}/{}", options.family, options.name));
        fs::write(&path, rewritten).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn validate_generated_plugin(target: &Path) -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["run", "-p", "amigo-plugin-check", "--", "validate", "--plugins"])
        .arg(target)
        .status()
        .map_err(|error| format!("failed to start plugin-check: {error}"))?;
    if status.success() { Ok(()) } else { Err(format!("plugin-check exited with {status}")) }
}

struct PluginNewOptions {
    family: String,
    name: String,
    kind: String,
    renderable: bool,
}

impl PluginNewOptions {
    fn from_args(args: &[String]) -> Option<Self> {
        let mut family = None;
        let mut name = None;
        let mut kind = None;
        let mut renderable = None;
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--family" => { index += 1; family = args.get(index).cloned(); }
                "--name" => { index += 1; name = args.get(index).cloned(); }
                "--kind" => { index += 1; kind = args.get(index).cloned(); }
                "--renderable" => { index += 1; renderable = args.get(index).and_then(|value| value.parse().ok()); }
                _ => return None,
            }
            index += 1;
        }
        let family = family?;
        let name = name?;
        let kind = kind?;
        if !valid_slug(&family) || !valid_slug(&name) { return None; }
        let renderable = renderable.unwrap_or(kind == "renderable-source");
        Some(Self { family, name, kind, renderable })
    }
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn copy_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() { copy_dir(&source_path, &target_path)?; } else { fs::copy(&source_path, &target_path)?; }
    }
    Ok(())
}
