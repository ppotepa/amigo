# Canonical Plugin Tree

Every Amigo domain plugin uses the same folder shape.

```txt
plugins/<family>/<plugin>/
  plugin.toml
  README.md

  src/
    lib.rs
    plugin.rs
    manifest.rs

    api/
      mod.rs
      ids.rs
      roles.rs
      capabilities.rs
      participation.rs
      contribution.rs
      response.rs
      coverage.rs
      candidate.rs
      targets.rs
      diagnostics_model.rs

    scene/
      mod.rs
      document.rs
      commands.rs
      hydration.rs
      validation.rs
      defaults.rs

    participation/
      mod.rs
      registry.rs
      adapters/
        mod.rs

    runtime/
      mod.rs
      service.rs
      collect.rs
      resolve.rs
      extract.rs
      resources.rs
      frame_state.rs

    render-wgpu/
      mod.rs
      buffers.rs
      targets.rs
      pass.rs
      pipelines.rs
      fallback.rs
      noop.rs

    scripting/
      mod.rs
      bindings.rs
      setters.rs
      getters.rs
      noop.rs

    diagnostics/
      mod.rs
      format.rs
      commands.rs
      snapshot.rs

  tests/
    hydration_tests.rs
    participation_tests.rs
    candidate_tests.rs
    waterfall_tests.rs
    diagnostics_tests.rs

  docs/
    pipeline.md
    contributions.md
    examples.md
    diagnostics.md
```

## Rules

* A plugin without rendering still keeps `src/render-wgpu/noop.rs`.
* A plugin without scripting still keeps `src/scripting/noop.rs`.
* A plugin without contributions still declares empty contribution lists in `plugin.toml`.
* Source plugins do not execute effects owned by another domain.
* Consumer plugins do not depend on concrete source implementations.
