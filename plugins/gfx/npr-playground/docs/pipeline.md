# `nprpipeline`

`nprpipeline` is the renderer-neutral drawing planner. The playground owns
only viewer state and a selected profile; it never decides how a mesh becomes
a stroke.

```text
scene mesh + camera
  -> SurfaceStrategy       topology / reusable surface facts
  -> FeatureStrategy       candidates: silhouette, boundary, structural and suggestive contours
  -> SalienceStrategy      which candidates deserve a mark
  -> ValueStrategy         paper-space value masses
  -> MarkStrategy          logical, stable marks
  -> StrokeChainStrategy   joins projected segments into one drawable path
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
| `PencilAnimation` | no fill (paper first) | silhouette + structural/suggestive contours, then selective | chained per-frame graphite redraw + overshoot | warm paper | graphite |

The neutral packet contains no WGPU handles. The WGPU backend chooses its ink
or graphite deposition shader from the packet's `NprMedium`. That keeps future
charcoal, marker and paint implementations additive: introduce a neutral
medium and a strategy/backend realization, without teaching scenes or the core
renderer about a particular mod.

`PencilAnimation` begins with no fill and no hatching grid: a studio
line-animation drawing is first judged by its contour. It chains visible
contour fragments before gesture realization, so one drawn path is not a
collection of mesh-edge dashes. Its paths redraw at 60 Hz with deterministic,
correlated drift and endpoint overshoot; material deposition evolves at 12 Hz.

The graphite backend uses paper tooth, fibres, a slow pressure drift and the
neutral medium's `filament_count` / `filament_spread`. These affect only
deposition, never feature selection or path identity. Next steps are sparse
tone masses, true lost-and-found segmentation, then selective surface-space
form-hatching bundles.
