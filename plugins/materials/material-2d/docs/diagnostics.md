# Material 2D Diagnostics

Channels:
- `material-2d.contributions`

## What To Trace
- Material id.
- Base color and opacity.
- Candidate status and target ids.
- Camera optics enabled state and intensity.
- Derived focus-depth role and scene-depth routing.

Opacity-zero materials should report a disabled optical response instead of
disappearing from contribution diagnostics.
