# Debug Views Diagnostics

Channels:
- `postfx.debug-views`

## What To Trace
- Selected debug view id.
- Available input targets.
- Missing targets requested by the selected view.
- Output `DiagnosticsSnapshot` entry.
- Target owner or producing plugin when that metadata is available.

Diagnostics should show why a view is empty without asking the renderer to guess
which domain produced the source data.
