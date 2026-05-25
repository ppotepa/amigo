# Focus Depth Diagnostics

Primary channel: `focus-depth.candidates`.

## Candidate Trace
The formatter reports owner, coverage label, candidate status, reason, and target
ids. Empty candidate lists are reported as `focus_depth.candidates: none`.

## Coverage Labels
- `scene_depth`
- `render_layer`
- `scene_object`
- `distance`
- `unsupported`

Diagnostics should identify the source plugin that failed to provide usable
coverage rather than blaming the render backend.
