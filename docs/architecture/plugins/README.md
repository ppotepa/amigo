# Amigo Plugin Architecture

Amigo uses domain-first plugin folders with layer-first dependencies.

A plugin is not just a package of code. A plugin owns a complete semantic waterfall:

Source
-> Roles / Capabilities
-> Contribution
-> Response
-> Coverage
-> Candidate
-> Target
-> Consumer
-> Final Output
-> Diagnostics
-> Tests

Core crates define stable contracts.
Plugin folders define domain implementation.
Renderer crates execute render work.
Apps compose plugins.
Mods provide content.

## Hard rules

- No old bridge.
- No retired compatibility wrappers.
- No `numbered duplicate` naming.
- No renderer-side domain guessing.
- No effect fallback from missing contribution.
- No app-owned domain logic.
- No global scene component bag.
- No global scripting binding bag.

## Required plugin files

Every plugin must eventually contain:

- `plugin.toml`
- `README.md`
- `src/plugin.rs`
- `src/api/`
- `src/scene/`
- `src/participation/`
- `src/runtime/`
- `src/render_wgpu/`
- `src/scripting/`
- `src/diagnostics/`
- `tests/waterfall_tests.rs`
- `docs/pipeline.md`
