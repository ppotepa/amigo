# Camera Optics Diagnostics

Channels:
- `camera.optical.contributions`
- `camera.optical.candidates`
- `camera.optical.targets`

## Candidate Trace
The candidate formatter reports owner, component kind, coverage kind, status,
reason, coverage details, render layer, color, intensity, target buffers, and
optical response values.

## Target Trace
- `scene_highlight` and `scene_emissive` are listed only for active candidates
  whose roles target those buffers.
- Empty candidate lists are reported explicitly.
- Unsupported coverage keeps its reason so the missing contribution can be fixed
  at the source plugin.
