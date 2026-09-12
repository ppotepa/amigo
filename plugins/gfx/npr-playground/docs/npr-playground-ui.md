# NPR Drawing Studio

## Document and migration

Drawing Studio is a single-model illustration workflow. Selecting a new source
model checkpoints a changed document as a local Draft, then opens a clean
document for the selected model. A Draft contains the camera, paper, palette,
ordered layers and exact pinned brush references. Invalid or missing brush
versions reject loading; the runtime never silently substitutes a brush.

The authored look format has one ordered `layers` list. Older split paint and
stroke documents must be migrated with the NPR migration workflow before they
are loaded. Runtime documents do not accept the old split representation.

## Workspace

```text
Sources + Brush library | paper viewport | Layers + Layer Inspector
                         Drafts / Variants
```

The Brush library is deliberately geometry-first: choose a source (`Contour`,
`Crease`, `Form lines`, `Shadow hatch`, `Flat fill`, or `Wash`), choose a
compatible brush preset, then add a layer. Applying a chosen brush to an
existing layer pins its id and version. The library only offers Stroke brushes
for line sources and Surface brushes for fill and wash sources.

The right-hand layer stack is flat and ordered. It supports visibility, lock,
duplicate, delete, move and Solo. Paper is a special document layer and cannot
be turned into a brush layer. Layer Inspector edits opacity, `Normal`,
`Multiply` and `Screen`; hatch layers also expose density and spacing; paint
layers expose wash and granulation. Layer diagnostics report extracted source
geometry, generated marks, mask coverage and a concrete no-effect reason.

`Coverage` uses the renderer feature-class diagnostic view; per-layer source
and mask coverage are reported in Layer Inspector diagnostics. `Build-up` is a
temporary compositor preview and Solo temporarily isolates paper plus one
layer; neither changes the stored layer stack.

Presets contain paper, palette and complete reusable layer compositions. Drafts
and variants are available below the viewport. Save writes the current drawing
document; Undo and Redo operate on authored document changes.

## Renderer contract

`NprGeometrySource` describes model-derived geometry independently from the
render medium. The extractor prepares source geometry once per model, then each
layer produces its own contribution and receives its stable `layer_id`.
Changing opacity, colour, blend, target or mask does not rebuild source paths;
changing a brush only invalidates dependent layer geometry. WGPU composites
contributions in global layer order and does not infer NPR intent from model or
object names.

Native GPU is used when available. JPEG is the presentation fallback; Local
RGBA is an internal transport path, not an authoring option.
