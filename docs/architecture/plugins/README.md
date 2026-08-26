# Amigo Plugin Architecture

Amigo uses domain-first plugin folders with layer-first dependencies.

A plugin is not just a package of code. A plugin owns a semantic waterfall:

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

- No retired compatibility wrappers.
- No numbered duplicate naming.
- No renderer-side domain guessing.
- No effect fallback from missing contribution.
- No app-owned domain logic.
- No global scene component bag.
- No global scripting binding bag.

## Enforced plugin structure

`amigo-plugin-check` requires every plugin to contain:

- `plugin.toml`
- `Cargo.toml`
- `README.md`
- `src/plugin.rs`
- `src/api/`
- `src/scene/`
- `src/runtime/`
- `src/scripting/`
- `src/diagnostics/`
- one render boundary: `src/render_wgpu/` for backend-specific work or `src/render/` for backend-neutral render adaptation
- `tests/waterfall_tests.rs`

Manifest-referenced documentation and tests must exist. A pipeline document is required by manifest validation.

## Participation directory

`src/participation/` is required by design for source/consumer plugins that explicitly model coverage, candidates, contributions, or target consumption. Tooling, bundle, adapter, and no-op plugins may omit it when those stages are not part of their contract. This conditional rule is why `plugin-check` does not impose the directory blindly on every plugin kind.
