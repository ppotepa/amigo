# char-3d Rust implementation

Native Rust port of `../strokes.html`.

The application opens two windows:

- `char-3d controls` - egui control surface mirroring the HTML UI layers.
- `char-3d renderer` - wgpu render window driven by the current control state.

## Run

From this directory:

```powershell
cargo run
```

From `tools/char-3d`:

```powershell
cargo run --manifest-path rust-impl\Cargo.toml
```

Headless end-to-end validation:

```powershell
cargo run --manifest-path rust-impl\Cargo.toml -- --self-test
```

By default the self-test writes SVG/PNG outputs and `all_models_atlas.png` to `%TEMP%\char-3d-rust-self-test`.

## Controls

- Drag in renderer window: orbit model or look around in freelook mode.
- Mouse wheel: zoom in orbit mode, dolly camera in freelook mode.
- `W/A/S/D`: move freelook camera forward/left/back/right.
- `Q/E`: move freelook camera down/up.
- Arrow keys: look around in freelook mode.
- `Shift`: faster freelook movement.
- `Space`: toggle auto rotate / FBX playback.

## Validate

```powershell
cargo fmt --check
cargo check
cargo test
```

or from `tools/char-3d`:

```powershell
cargo fmt --manifest-path rust-impl\Cargo.toml --check
cargo check --manifest-path rust-impl\Cargo.toml
cargo test --manifest-path rust-impl\Cargo.toml
cargo clippy --manifest-path rust-impl\Cargo.toml -- -D warnings
cargo run --manifest-path rust-impl\Cargo.toml -- --self-test
```

## Scope

Implemented:

- native egui control window and separate wgpu renderer window
- built-in OBJ assets and binary FBX geometry loading
- built-in `walking` playback from `assets/models/walking.amc`, a baked browser/Three.js vertex animation clip
- raw custom FBX fallback with a native Rust geometry deformation pass
- playback clock wired to `anim_fps`, imprecise tweening, and frame-dependent NPR seeds
- CPU frame pipeline for projection, contours, marks, paint regions, lighting, and cleanup/debug controls
- SVG, PNG, and atlas export paths
- persisted control state under the user config directory

Bake the browser-derived clip from `tools/char-3d`:

```powershell
npm run bake:walking
```

Current limitation:

- Rust does not implement full skeletal FBX skinning/mixer parsing. Built-in `walking` uses the baked `.amc` clip; raw custom FBX remains a fallback path.
