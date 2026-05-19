# Plugin Manifest

Every plugin has a `plugin.toml`.

The manifest is consumed by:

- plugin registry
- slot registry
- diagnostics
- codemap
- audit-build
- future mini-LSP

## Required shape

```toml
id = "amigo.family.plugin-name"
family = "family"
kind = "renderable-source"
renderable = true

[capabilities]
provides = []
requires = []

[slots]
implements = []
requires = []
replaces = []

[targets]
reads = []
writes = []
contributes = []

[contributions]
emits = []
consumes = []

[diagnostics]
channels = []

[docs]
pipeline = "docs/pipeline.md"
contributions = "docs/contributions.md"
diagnostics = "docs/diagnostics.md"

[tests]
hydration = "tests/hydration_tests.rs"
participation = "tests/participation_tests.rs"
candidate = "tests/candidate_tests.rs"
waterfall = "tests/waterfall_tests.rs"
diagnostics = "tests/diagnostics_tests.rs"
```

## Plugin kinds

```txt
renderable-source
semantic-source
target-consumer
source-and-consumer
bundle
adapter
tooling
noop
```

## Rules

* `id` is globally unique.
* `family` must match the folder under `plugins/`.
* `kind` defines plugin responsibility.
* `replaces` is allowed only through explicit slots.
* Missing optional plugin behavior must be declared as `noop`, not inferred.
