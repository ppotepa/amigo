# Amigo

Mod-first Rust engine + tooling monorepo (runtime + launcher + editor).

## Co jest co (skrót)

```text
crates/
  apps/
    app/                     ─ serwer/aplikacja runtime (game runtime)
    launcher/                ─ TUI launcher uruchamiający app
    amigo-editor/            ─ desktopowy editor (Tauri + Vite)

  foundation/                ─ core podstawowe typy/abstrakcje
  engine/                    ─ scena, runtime, assets, render, input, audio
  platform/                  ─ host back-endy (winit, file watch, windows)
  scripting/                 ─ scripting API i adaptery (Rhai)
  2d/                       ─ systemy renderowania/komponenty 2D
  3d/                       ─ systemy renderowania/komponenty 3D
  ui/                       ─ wspólne widgety/UI runtime
  audio/                    ─ audio API/implementacje
  tools/                    ─ utily deweloperskie

mods/
  core, core-game, playground-2d, playground-3d, ...
```

`mods/*` to runtime content:

- `scene.yml` + `scene.rhai` definiują sceny
- `assets/*` to zasoby i assety
- `mod.rhai` to opcjonalna logika modułu

## Jak uruchomić

### 0) Przygotowanie

```powershell
# raz na świeżo
cargo build --workspace
```

### 1) App (runtime) przez launcher

```powershell
# TUI launcher: wybór profilu i sceny z config/launcher.toml
cargo run -p amigo-launcher

# bezpośredni start (hosted + szybki start sceny)
cargo run -p amigo-launcher -- --hosted --mod=playground-2d --scene=basic-scripting-demo
```

Opcje:

- `--mod=<mod-id>` — root mod (np. `playground-2d`, `core-game`, `core`)
- `--scene=<scene-id>` — scena startowa (`basic-scripting-demo`, `hello-world-cube`…)
- `--headless` — uruchomienie bez okna (tryb konsolowy)
- `--profile=<id>` — profil z `config/launcher.toml` (`dev`, `release`, itp.)

### 2) App bez launchera (bezpośrednio)

```powershell
cargo run -p amigo-app -- --hosted --mods-root mods --mod=playground-2d --scene=basic-scripting-demo
```

### 3) Editor

```powershell
cd crates/apps/amigo-editor
npm install
npm run tauri:dev      # pełny tryb desktop (Tauri)

# albo tylko frontend:
npm run dev
```

## Co testować w praktyce

- Najpierw sprawdź działanie `app`:
  - launcher TUI (`cargo run -p amigo-launcher`)
  - szybki profil hostowany 2D (`--mod=playground-2d --scene=basic-scripting-demo`)
- Potem editor:
  - `npm run tauri:dev` z poziomu `crates/apps/amigo-editor`

## Klucze do eksploracji kodu

- `crates/apps/launcher/src/main.rs` — argumenty launchera (`--mod`, `--scene`, `--profile`)
- `crates/apps/app/src/main.rs` — API uruchomienia runtime (`BootstrapOptions`, `--hosted`, `--scene`)
- `config/launcher.toml` — profile startowe
- `crates/engine/scene/` — model sceny i komponenty
- `crates/engine/runtime/` — przebieg runtime
- `crates/engine/render-wgpu/` — backend rendera
- `crates/apps/amigo-editor/src` — kod frontu edytora

## Dodatkowe docs

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/RHAI_API.md](docs/RHAI_API.md)
