# Shutter Motion Diagnostics

Primary channel: `shutter-motion.candidates`.

## Candidate Trace
The formatter reports owner, coverage label, status, reason, and target ids.
Empty candidate lists are reported as `motion_shutter.candidates: none`.

## Coverage Labels
- `scene_velocity`
- `camera_motion`
- `render_layer`
- `unsupported`

Motion runtime debugging should stay separate from shutter target diagnostics:
controller state explains movement, while candidate diagnostics explain render
participation.
