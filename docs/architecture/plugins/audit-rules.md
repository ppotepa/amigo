# Plugin Architecture Audit Rules

## Required file rules

- Every plugin must have `plugin.toml`.
- Every plugin must have `README.md`.
- Every plugin must have `docs/pipeline.md`.
- Every plugin must have `tests/waterfall_tests.rs`.
- Every plugin must have `src/plugin.rs`.
- Every plugin must have `src/api/mod.rs`.
- Every plugin must have `src/diagnostics/mod.rs`.

## Forbidden pattern rules

- No `legacy`.
- No `deprecated`.
- No `_v2`.
- No `luma_fallback`.
- No `guess_optical`.
- No `direct_lens_flare`.
- No renderer-side `should_produce_scene_highlight`.

## Ownership rules

- Source plugin cannot execute consumer effect.
- Consumer plugin cannot import source internals.
- Renderer cannot synthesize domain contribution.
- App cannot import plugin internals outside composition.
- Scene crate cannot own plugin descriptors.
- Scripting host cannot own plugin-specific bindings.

## Codemap rules

- Every plugin manifest must index into CodeMapGraph.
- Every diagnostic channel must map to owning plugin.
- Every target write must map to owning plugin.
- Every waterfall must have at least one test.
