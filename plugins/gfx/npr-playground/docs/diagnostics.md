# NPR playground diagnostics

The plugin publishes `RenderFrameStats` through the existing render diagnostics
service. Relevant values include geometry, topology edges, feature segments,
silhouettes, creases, strokes, stroke vertices/indices, hatching, construction
marks and stroke-budget rejections.

The workshop exposes `Final`, `FeatureClasses` and `StrokeIds` debug views.
These are resolved in the NPR domain packet, not by renderer-side inspection.

The panel reports FPS and frame time for observation only. Stroke redraw cadence
is controlled by `NprMotionPolicy`, not by display FPS. `Pause sketch` affects
gesture variation independently from object animation playback.
