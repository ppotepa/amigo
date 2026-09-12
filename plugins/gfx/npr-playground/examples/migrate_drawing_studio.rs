//! Converts one explicitly chosen subject from a retired multi-model profile.
//! Preview is the default; `--apply` writes only after every input is prepared.
use amigo_npr_playground_plugin::documents::migration::DrawingStudioProfileMigration;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--help" || arg == "-h") {
        println!("usage: migrate_drawing_studio [--apply] <source-model> <profile.yml> ...");
        return Ok(());
    }
    let apply = args.first().is_some_and(|arg| arg == "--apply");
    if apply {
        args.remove(0);
    }
    if args.len() < 2 {
        return Err(
            "usage: migrate_drawing_studio [--apply] <source-model> <profile.yml> ...".into(),
        );
    }
    let source_model = args.remove(0);
    // Prepare every file first: an invalid later profile must not leave an
    // earlier profile migrated without the user having reviewed all inputs.
    let migrations = args
        .iter()
        .map(|arg| {
            let path = PathBuf::from(arg);
            let migration = DrawingStudioProfileMigration::preview(path.clone(), &source_model)?;
            let removed = if migration.removed_models.is_empty() {
                "no other models".into()
            } else {
                format!("removes {}", migration.removed_models.join(", "))
            };
            println!("{}: keeps {} ({removed})", path.display(), migration.source_model);
            let backup = PathBuf::from(format!("{}.before-drawing-studio", path.display()));
            if apply && backup.exists() {
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
