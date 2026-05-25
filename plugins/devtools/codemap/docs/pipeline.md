# Codemap Pipeline

Codemap owns the tooling path that turns repository index data into devtools
diagnostics.

## Flow
- The codemap index provider reads repository metadata and symbol ownership
  information.
- The diagnostics provider reads the current `DiagnosticsSnapshot`.
- Codemap appends index-oriented diagnostics and writes the updated snapshot.
- Devtools consumers use that snapshot for navigation and review support.

## Targets
- Reads `DiagnosticsSnapshot`.
- Writes `DiagnosticsSnapshot`.
- Does not participate in render extraction or backend target execution.
