# Camera Core Diagnostics

Primary channel: `camera-core.binding`.

## Runtime Signals
- Focus target summaries are available through `CameraFocusTarget2dService` as
  `camera.focus.targets`.
- Focus target resolution records unknown, stale, or ambiguous selectors as the
  service last error.
- Runtime control exposes camera state through the camera control provider when
  the runtime control service is present.

## What To Check
- Camera frame providers should be registered before focus depth, shutter motion,
  or camera optics consumers run.
- Missing focus targets should be reported by selector, not silently replaced by
  a default target.
- Diagnostics should describe the camera binding state without reading renderer
  internals.
