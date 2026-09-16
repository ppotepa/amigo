# `nprpipeline`

`nprpipeline` is the renderer-neutral drawing planner. The playground owns
only viewer state and a selected profile; it never decides how a mesh becomes
a stroke.

```text
scene mesh + camera
  -> SurfaceStrategy       topology / reusable surface facts
  -> FeatureStrategy       candidates: silhouette, boundary, crease
  -> SalienceStrategy      which candidates deserve a mark
  -> ValueStrategy         paper-space value masses
  -> MarkStrategy          logical, stable marks
  -> GestureStrategy       physical paths and pressure-ready ribbons
  -> PaperStrategy         paper/background response
  -> MediumStrategy        neutral Ink | Graphite packet selection
  -> TemporalState         independent path-redraw / material-boil epochs
  -> NprRenderPacket
  -> WGPU medium pipeline  ink stroke or graphite deposition
```

Every stage is a `Send + Sync` trait. A profile is a composition of these
strategies, not a second renderer:

| Profile | Value | Salience | Gesture | Paper | Medium |
| --- | --- | --- | --- | --- | --- |
| `MinimalInk` | three-band | all contours | flat | flat | ink |
| `PencilAnimation` | no fill (paper first) | selective contours | per-frame graphite redraw + overshoot | warm paper | graphite |

The neutral packet contains no WGPU handles. The WGPU backend chooses its ink
or graphite deposition shader from the packet's `NprMedium`. That keeps future
charcoal, marker and paint implementations additive: introduce a neutral
medium and a strategy/backend realization, without teaching scenes or the core
renderer about a particular mod.

The current graphite shader is intentionally the first vertical slice: stable
paper tooth and pigment breakup. `PencilAnimation` begins with no fill and no
hatching grid: a studio line-animation drawing is first judged by its contour.
Its paths redraw at 60 Hz with deterministic, correlated drift and endpoint
overshoot; material deposition evolves at 12 Hz. Next steps are sparse tone
masses, lost-and-found segmentation, then selective form-hatching bundles.
