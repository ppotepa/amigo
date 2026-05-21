# apps-launcher

Path: `crates/apps/launcher`  
Cargo package: `amigo-launcher`  
Layer: launcher

## Role

Launcher/profile selection surface. Should choose runtime presets, not own domain behavior.

## Owns

- local crate-owned contracts
- tests for crate-owned behavior
- small targeted fixes inside ownership boundary

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner

## Important files found in snapshot

- `crates/apps/launcher/src/config.rs`
- `crates/apps/launcher/src/diagnostics.rs`
- `crates/apps/launcher/src/main.rs`
- `crates/apps/launcher/src/tui/details.rs`
- `crates/apps/launcher/src/tui/discovery.rs`
- `crates/apps/launcher/src/tui/filtering.rs`
- `crates/apps/launcher/src/tui/profiles.rs`
- `crates/apps/launcher/src/tui/render.rs`
- `crates/apps/launcher/src/tui/runtime.rs`
- `crates/apps/launcher/src/tui/state/config.rs`
- `crates/apps/launcher/src/tui/state/expansion.rs`
- `crates/apps/launcher/src/tui/state/input.rs`
- `crates/apps/launcher/src/tui/state/profile.rs`
- `crates/apps/launcher/src/tui/state/tree.rs`
- `crates/apps/launcher/src/tui/state.rs`
- `crates/apps/launcher/src/tui/tests.rs`
- `crates/apps/launcher/README.md`

## Dependencies seen in Cargo.toml

- `amigo-app`
- `amigo-core`
- `amigo-modding`
- `crossterm`
- `ratatui`
- `serde`
- `toml`

## Documentation status

README present: `true`

If this crate is touched, keep documentation close to the touched ownership boundary. Do not use this crate doc as permission to perform broad cleanup.

## Allowed changes

```text
small changes inside crate ownership
contract changes with downstream validation
local tests for crate-owned behavior
diagnostics that expose missing contracts or invalid input
```

## Forbidden changes

```text
cross-layer behavior leaks
legacy/v2 duplicate paths
large formatting-only rewrites
new hidden fallback behavior
```

## Validation commands

```powershell
cargo check -p amigo-launcher
cargo test -p amigo-launcher --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/apps/launcher
rg -n "pub struct|pub enum|pub trait|impl " crates/apps/launcher/src
```
