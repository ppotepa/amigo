# Diagnostics

The packet exposes deterministic NPR geometry, feature, stroke and viewport counters.

`npr.render` and `npr.features` are declared manifest channels. RenderFrameStats
aggregates geometry, topology edges, feature classes, stroke/vertex/index counts,
viewport and debug view through the existing renderer diagnostics path.
FeatureClasses colors classes in the NPR domain; StrokeIds colors assembled chains.

The external panel reads FPS and frame time as read-only metadata. These are
wall-clock measurements, excluded from presets and deterministic golden checks.
Invalid bindings/layouts and preset errors appear in the panel; malformed hot
reloads retain the last valid layout. No full per-frame report is logged in release.

The Diagnostics tab exposes aggregate geometry, topology, feature-segment and
stroke/vertex/index counts plus viewport dimensions from the last RenderExtract.
The aggregate also keeps silhouette and crease counts, while tone lines are
included in the stroke count because they use the same deterministic stroke
contract. `smooth_contour_spans` separates normal-field contours from
`feature_segments`, which count only topology-edge features. `gesture_variant_epoch` is the maximum domain-selected redraw epoch in
the extracted view; zero means Stable or no qualifying surface motion. It is a
diagnostic, not a wall-clock counter. Debug views are resolved before extraction: `Final` shows the selected
tool response, `FeatureClasses` shows domain classification, and `StrokeIds`
shows stable assembled-chain identities.

`surface_source_triangles` and `surface_proxy_triangles` expose the exact cost
of the selected Smooth proxy. A differing pair is an authored surface-policy
decision, not an implicit renderer LOD.

`feature_candidates` counts post-surface-policy topology candidates and
`feature_rejected` counts segments belonging to crease chains omitted for being
shorter than the configured pixel threshold. Boundary and silhouette chains are
not affected by this ranking pass.

`hatching_confidence_rejected` counts tonal paths removed before tessellation
because their local tangent is ambiguous or their normal field turns too
abruptly. Adjust `min_form_line_confidence` in the Tone panel to trade local
detail for a cleaner, less triangulation-driven drawing.

`suggestive_contour_spans` reports opt-in radial-curvature form lines. They
use a separate stable identity range from silhouettes and are emitted only on
Smooth surfaces when `suggestive_contours` is enabled.

Suggestive contours and tonal form-lines have independent width/opacity
controls. Width is applied before tessellation; opacity scales the deterministic
coverage field afterwards, so neither control reclassifies geometry or changes
the renderer's pass ordering.

`construction_marks` and `construction_rejected` are reserved for editor or
plugin-authored strokes resolved from source-surface anchors. A stale anchor
revision is rejected atomically before a packet is mutated.

`stroke_budget_rejected` and `stroke_budget_exhausted` report a deterministic
CPU packet limit before WGPU upload. Feature strokes are retained before tonal
strokes, so a dense scene degrades predictably instead of allocating an invalid
GPU buffer or silently dropping an arbitrary batch.
The footer keeps FPS/frame time visible. Thumbnails are fixed neutral references;
W/R/S badges reflect current visibility, effective rotation and style override.
Zoom regression tests cover monotonic/reversible motion, 20–240 FPS equivalence,
fractional wheel input, bounds, pause, fit and simulation catch-up without replaying
one wheel event. Smoothing cannot create additional rendered frames at low GPU FPS.
