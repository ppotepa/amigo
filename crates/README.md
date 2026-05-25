# Crates

Crate inventory is generated from Cargo metadata and codemap, not maintained as
one markdown file per crate.

Use targeted navigation before editing a crate:

```powershell
cargo build -p amigo-codemap
$cm = "target\debug\amigo-codemap-stable.exe"
& $cm change-plan "<task>" --limit 20
& $cm open-set "<crate or symbol>" --why --limit 20
```

For ownership rules, use `AGENTS.md` and `PROJECT.md`. For current package
names and dependencies, use each crate's `Cargo.toml`.
