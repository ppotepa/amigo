# Codemap Contributions

Codemap is a devtools plugin, not a render or gameplay source.

## Provides
- `devtools.codemap.index@1` for repository index data.
- `devtools.diagnostics.provider@1` for diagnostics snapshot integration.
- Slots `codemap.index_provider` and `diagnostics.provider`.

## Consumes
- Existing `DiagnosticsSnapshot` entries when the tooling layer enriches or
  republishes index diagnostics.

Codemap emits no render targets and no domain visual contributions. Its output
is diagnostics data that helps locate symbols, ownership, and generated-file
boundaries.
