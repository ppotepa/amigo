use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        eprintln!(
            "usage: amigo-plugin-new <family> <plugin-name> <kind> <renderable:true|false>"
        );
        std::process::exit(2);
    }

    let family = &args[1];
    let plugin_name = &args[2];
    let kind = &args[3];
    let renderable = &args[4];

    let source = PathBuf::from("templates/plugin");
    let target = PathBuf::from("plugins").join(family).join(plugin_name);

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
            &format!("amigo.{family}.{plugin_name}"),
        )
        .replace("family = \"family\"", &format!("family = \"{family}\""))
        .replace("kind = \"noop\"", &format!("kind = \"{kind}\""))
        .replace("renderable = false", &format!("renderable = {renderable}"));

    fs::write(manifest_path, manifest).unwrap();

    println!("{}", target.display());
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
