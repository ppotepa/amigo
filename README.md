# Amigo

Amigo is a modular Rust game engine workspace built around a mod-first runtime model.

Current foundation includes:

- `launcher` TUI for selecting a root mod and scene
- `app` runtime bootstrap with hosted and headless modes
- scene-centric content layout: `scene.yml` + `scene.rhai`
- optional persistent `mod.rhai`
- `Rhai` scripting through a domain-based `world.*` API
- 2D and 3D playground mods
- hot reload, file watching, and basic `wgpu` rendering paths

## Workspace shape

```text
crates/
  foundation/
  engine/
  platform/
  scripting/
  2d/
  3d/
  apps/

mods/
  core/
  core-game/
  playground-2d/
  playground-3d/
```

## Running

Launcher:

```powershell
cargo run -p amigo-launcher
```

Hosted app directly:

```powershell
cargo run -p amigo-app -- --hosted --mod=playground-2d --scene=basic-scripting-demo
```

## Architecture

See:

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/RHAI_API.md](docs/RHAI_API.md)
