# Trails 2D Diagnostics

Channels:
- `trails-2d.render`
- `trails-2d.contributions`

## What To Trace
- Trail id.
- Render layer and length in pixels.
- Candidate status and target ids.
- Scene color and alpha write targets.
- Camera optics role routing.
- Shutter motion coverage and computed motion blur.

Diagnostics should keep zero-length trails visible as disabled contribution
sources rather than dropping them from the trace.
