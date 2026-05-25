# Shutter Motion Contributions

Shutter Motion declares `MotionShutterContribution2d` with
`DisabledByDefault` policy.

## Inputs
- Sprite and particle sources can provide render-layer motion coverage.
- Animated beacon sources can provide camera motion coverage.
- Coverage kinds include `SceneVelocity`, `CameraMotion`, render layer, and
  unsupported coverage with a reason.

## Targets
- `SceneVelocity` carries source velocity.
- `TemporalExposure` carries shutter accumulation.
- `FinalComposite` is the final target when the shutter pass participates in
  presentation.

Candidates must be active and supported before they affect target planning.
