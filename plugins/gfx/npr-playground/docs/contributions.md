# NPR playground contributions

Scene, model, included look and object look patches share the same authored
`layers` contract. Paint and strokes no longer occupy separate document lists.
The per-layer `parameters.paint` field still describes wash/granulation; it is
not a layer collection and the migration preserves it unchanged. Repeated
generator kinds can be represented with distinct IDs; independent brush
generation and versioned brush resources are explicit domain contracts. Brush
versions are immutable; instances hold local overrides, targets and masks.
Groups apply opacity/blend over their children, and UpdateBrushVersion is one
history operation.

The plugin contributes the `gfx.npr` capability, an `NprSettings` scene
component, the `npr-playground` provider for typed intents/snapshots, and one
render extractor contribution per visible object.

Styles resolve in this order:

```text
engine defaults -> model defaults -> included looks -> active look
  -> scene profile -> object override -> live session patch
  -> effective NprDrawCommand
```

Layer scalar overrides are sparse (`enabled`, `opacity`, `blend`, colour source
and tool). A local reorder is explicitly structural and records the full stable
layer-ID order. Therefore later scene changes still reach object layers that did
not override the relevant property.

For stroke layers, an explicit tool is applied by `amigo-render-npr` after
feature extraction and before backend submission. It retargets the pixel-space
strip envelope, analytic edge softness, coverage and round caps while retaining
the source stroke identity and topology. `inherit` keeps the scene `ComicInk`
tool.

`ThreeBand` shading can also emit sparse `FormLine` marks. They are distinct
from `Tone`/hatching in the render packet, diagnostics, budget priority and
the ordered `form-lines` layer; disabling that layer never removes hatching.

`Underpainting` receives its own packet channel rather than reusing flat fill
triangles. Each source face carries restrained seeded pigment coverage, so the
layer can be reordered, disabled or blended without changing `Fill`.
Its typed medium exposes `wash` (0..2) and `granulation` (0..1); both are
resolved sparsely from scene to object. Wash is applied to packet coverage;
granulation is sampled continuously by the dedicated paint material, avoiding
triangle-boundary artifacts.
The separate `fill` layer is disabled by default and can explicitly place the
crisp three-band geometry over the wash.

`Watercolour Wash` is a built-in typed look, not a renderer preset. It selects
the brush and three-band response, enables an underpainting with `wash: 1.12`
and `granulation: 0.64`, and disables flat fill and hatching. Applying it in
object scope produces sparse layer overrides; applying it in scene scope
replaces the scene layer stack.

WGPU executes `normal`, `multiply` and `screen`. `overlay` is intentionally not
available in the workshop until destination-sampling compositing exists.
`Constant` and `ModelBaseColor` colour sources are executed for fill/stroke
layers. The latter uses the selected object's explicitly authored RGBA material
contribution, carried by `NprDrawCommand`; it never falls back to a palette.
