# Debug Views Pipeline

Debug Views turns declared render targets into devtools inspection output.

## Flow
- Scene data declares a `DebugViews` post-fx tooling component.
- Runtime selects the requested debug view target plan.
- The backend reads the selected render target.
- Debug output is written into `DiagnosticsSnapshot` for devtools consumers.

## Targets
- Reads scene color and other declared inspection targets.
- Writes `DiagnosticsSnapshot`.
- Does not alter final scene presentation.
