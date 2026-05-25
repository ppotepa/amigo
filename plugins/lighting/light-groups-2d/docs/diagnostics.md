# Light Groups 2D Diagnostics

Channels:
- `light-groups-2d.contributions`

## What To Trace
- Light group id.
- Candidate status and target ids.
- Optional lightmap source and channel.
- Camera optics coverage kind or unsupported reason.
- Highlight and emissive role routing declared for the group.

Diagnostics should identify incomplete group declarations at the group source,
not later in the renderer.
