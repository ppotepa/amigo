# amigo-launcher

CLI/TUI launcher for Amigo profiles.

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
- amigo-core.
- amigo-modding.
- crossterm.
- ratatui.
