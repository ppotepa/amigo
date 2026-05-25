# Plugins

Plugins are organized by domain family:

```text
plugins/<family>/<plugin>/
```

A plugin owns its domain waterfall:

```text
source/document
  -> roles / capabilities
  -> contribution
  -> response
  -> coverage
  -> candidate
  -> target
  -> consumer
  -> diagnostics
  -> tests
```

Plugin-local documentation lives with the plugin:

```text
plugins/<family>/<plugin>/README.md
plugins/<family>/<plugin>/docs/pipeline.md
plugins/<family>/<plugin>/docs/contributions.md
plugins/<family>/<plugin>/docs/diagnostics.md
```

The old root-level `plugins/*-*.md` inventory snapshots are retired. Use codemap,
`plugin.toml`, and the plugin-local docs for current state.
