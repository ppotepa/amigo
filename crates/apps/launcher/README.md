# amigo-launcher

CLI/TUI launcher for Amigo profiles.

Scene-declared external panels work in dev hosted/TUI and release hosted modes.
The internal panel-client mode is dispatched before loading launcher config;
window and protocol implementation remain in the engine/UI crates.

For NPR Gallery:

```text
cargo run -p amigo-launcher -- --hosted --profile dev --mod npr-playground --scene gallery
```

Close a running launcher before rebuilding on Windows. Release profiles require
a freshly built `cargo build --release -p amigo-app`; there is no debug substitution.
Missing panels report `[panels.host]` in the host console and run logs. See
`docs/architecture/runtime-panels.md` for recovery and the launch-mode test matrix.

## Responsibility
- Load launcher config.
- Validate selected launch profile.
- Start hosted or headless runtime modes.
- Provide terminal profile selection.

## Not here
- Engine runtime implementation.
- Mod catalog internals.
- Renderer backend logic.

## Depends on
- amigo-app.
- amigo-runtime-bundles (shared host dispatch only).
- amigo-core.
- amigo-modding.
- crossterm.
- ratatui.
