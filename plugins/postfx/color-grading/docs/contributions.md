# Color Grading Contributions

Color Grading is a post-fx target consumer. It emits no scene-source
contributions.

## Consumes
- `SceneColor` as the image to grade.
- The active color grading target plan for final output routing.

## Routing
- `ColorGradingTargetPlan::final_composite()` reads `SceneColor`.
- The plan writes `FinalComposite`.
- Any authored grading profile is evaluated as post processing, after source
  plugins have already produced their visual targets.

Color grading should not change lighting, material, or optics ownership. It only
transforms the declared color input.
