//! Preview by default; --apply writes only after a durable exclusive backup.
use amigo_npr_playground_plugin::documents::migration::LayerStackMigration;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let apply = args.first().is_some_and(|arg| arg == "--apply");
    if apply {
        args.remove(0);
    }
    if args.is_empty() {
        return Err("usage: migrate_layer_stack [--apply] <document.yml> ...".into());
    }
    // Prepare every file first so a malformed later document aborts before writes.
    let migrations = args
        .iter()
        .map(|arg| {
            let path = PathBuf::from(arg);
            let migration = LayerStackMigration::preview(path.clone())?;
            println!(
                "{}: {}",
                path.display(),
                if migration.changes.is_empty() {
                    "unchanged"
                } else {
                    "conversion prepared"
                }
            );
            for change in &migration.changes {
                println!("  {change}");
            }
            let backup = PathBuf::from(format!("{}.before-layer-stack", path.display()));
            if apply && !migration.changes.is_empty() && backup.exists() {
                return Err(format!("backup already exists: {}", backup.display()));
            }
            Ok((migration, backup))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if apply {
        for (migration, backup) in migrations {
            migration.apply(&backup)?;
        }
    }
    Ok(())
}
