use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(options) = PluginNewOptions::from_args(&args) else {
        eprintln!("usage: amigo-plugin-new --family <family> --name <plugin-name> --kind <kind> [--renderable true|false]");
        std::process::exit(2);
    };

    let source = PathBuf::from("templates/plugin");
    let target = PathBuf::from("plugins")
        .join(&options.family)
        .join(&options.name);

    if target.exists() {
        eprintln!("target already exists: {}", target.display());
        std::process::exit(1);
    }

    copy_dir(&source, &target).unwrap();

    let manifest_path = target.join("plugin.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    let manifest = manifest
        .replace(
            "amigo.family.plugin-name",
            &format!("amigo.{}.{}", options.family, options.name),
        )
        .replace(
            "family = \"family\"",
            &format!("family = \"{}\"", options.family),
        )
        .replace("kind = \"noop\"", &format!("kind = \"{}\"", options.kind))
        .replace(
            "renderable = false",
            &format!("renderable = {}", options.renderable),
        );

    fs::write(manifest_path, manifest).unwrap();

    let cargo_path = target.join("Cargo.toml");
    let cargo = fs::read_to_string(&cargo_path).unwrap();
    let cargo = cargo.replace(
        "amigo-plugin-template",
        &format!("amigo-{}-plugin", options.name),
    );
    fs::write(cargo_path, cargo).unwrap();

    println!("{}", target.display());
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
                "--family" => {
                    index += 1;
                    family = args.get(index).cloned();
                }
                "--name" => {
                    index += 1;
                    name = args.get(index).cloned();
                }
                "--kind" => {
                    index += 1;
                    kind = args.get(index).cloned();
                }
                "--renderable" => {
                    index += 1;
                    renderable = args.get(index).and_then(|value| value.parse().ok());
                }
                _ => return None,
            }
            index += 1;
        }

        let family = family?;
        let name = name?;
        let kind = kind?;
        let renderable = renderable.unwrap_or(kind == "renderable-source");

        Some(Self {
            family,
            name,
            kind,
            renderable,
        })
    }
}

fn copy_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}
