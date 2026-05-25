# Trails 2D Pipeline

Trails 2D renders trail geometry and declares optional camera participation.

## Flow
- Scene data declares a `Trail2D` component.
- Runtime builds a `Trail2dSource` with id, render layer, and length.
- The source becomes an active candidate for scene color and alpha targets.
- Participation adapters emit camera optics and shutter motion records when the
  trail declares them.
- The backend renders the trail through the source-renderer path.

## Targets
- Writes `SceneColor` and `SceneAlpha`.
- Contributes `SceneVelocity`, `SceneHighlight`, and `SceneEmissive`.
- Does not consume post-fx or lighting outputs directly.
