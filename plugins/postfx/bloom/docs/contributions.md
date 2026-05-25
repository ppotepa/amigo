# Bloom Contributions

Bloom is a post-fx target consumer. It does not emit source contributions.

## Consumes
- Highlight and emissive target data declared by upstream source plugins.
- The active bloom target plan for read and write routing.

## Routing
- `BloomTargetPlan::standard()` reads `SceneHighlight` and `SceneEmissive`.
- The standard plan writes `CameraArtifactLayer`.
- Bloom sources must arrive through explicit highlight, emissive, or optics
  contributions before this pass runs.

Bloom should not decide that an object glows from names, components, or texture
presence alone.
