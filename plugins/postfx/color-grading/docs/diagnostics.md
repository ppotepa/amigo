# Color Grading Diagnostics

Channels:
- `postfx.color-grading`

## What To Trace
- Active target plan.
- Input `SceneColor` availability.
- Output target for the graded frame.
- Selected grading component or profile when authored data is present.
- Disabled or missing grading state.

Diagnostics should separate missing input color from an intentionally neutral
grade.
