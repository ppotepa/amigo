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

## Requirements

- Rust stable toolchain (`rustup default stable`)
- Node.js 20+ and `npm` (for `amigo-editor` frontend)
- Windows, Linux, or macOS environment supported by Tauri v2

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
  they-are-rotten/
  playground-2d-asteroids/
  playground-sidescroller/
```

## Quickstart

1. Clone and enter repo:

```powershell
git clone <repo-url>
cd amigo
```

2. (Editor only) install frontend deps:

```powershell
cd crates/apps/amigo-editor
npm install
cd ../../..
```

3. Run one of the apps below.

## Run: Editor (Tauri)

From workspace root:

```powershell
cargo run -p amigo-editor
```

What you should see:

- startup dialog with available mods
- create/open/delete project flows
- scene preview and project workspace

## Run: Launcher (TUI)

```powershell
cargo run -p amigo-launcher
```

## Run: Runtime App Directly

```powershell
cargo run -p amigo-app -- --hosted --mod=playground-2d --scene=basic-scripting-demo
```

You can replace `--mod` / `--scene` with any valid mod/scene id in `mods/*`.

## Architecture

See:

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/RHAI_API.md](docs/RHAI_API.md)
