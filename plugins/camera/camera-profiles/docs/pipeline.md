# Camera Profiles Pipeline

Camera Profiles owns named camera profile data and quality presets. It is a data
bundle for camera consumers, not a renderer or scene source.

## Flow
- Profile references resolve to `CameraProfile2d` entries with lens, film, and
  focus distance metadata.
- Quality presets map to `CameraQualitySettings2d` for preview, gameplay,
  cinematic, and debug use.
- Camera Core reads selected profile and quality state when building frame
  context.

## Boundaries
- No render extraction runs in this plugin.
- No render target is read or written.
- Backend quality decisions must be driven by the selected profile data, not by
  app-side wiring.
