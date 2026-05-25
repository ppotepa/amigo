# Codemap Diagnostics

Channels:
- `devtools.codemap.index`

## What To Trace
- Index source, package or plugin id, and repository path scope.
- Indexed symbol, file, and owner counts.
- Skipped generated artifacts and generated concat snapshots.
- Errors from unavailable or stale index data.

Codemap diagnostics should explain why a symbol is missing from the index
without changing runtime composition or render behavior.
