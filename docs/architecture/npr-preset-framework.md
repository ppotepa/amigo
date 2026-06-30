# NPR Preset Framework

Amigo NPR presets are authored data that resolve into a renderer-facing pipeline plan. The renderer executes that plan; it must not infer artistic intent from scene names, model names, or debug strings.

## Data Flow

```text
YAML preset / scene npr settings
  -> NprLineSettings3d
  -> NprPipelinePlan3d
  -> CPU reference or GPU realtime renderer
  -> RenderFrameStats diagnostics
```

`NprLineSettings3d::pipeline_plan()` is the canonical resolver. It preserves the authored strategy fields and adds validation warnings for incoherent combinations, for example `akira_ink` without `character_semantic` candidates.

## Strategy Layers

- `render_strategy` selects the backend route: `gpu_realtime` or `cpu_reference`.
- `candidate_strategy` selects the line candidate source policy.
- `path_strategy` selects direct segments or stable stroked paths.
- `stroke_strategy` selects the brush/stroke model.
- `fill_strategy` selects ink fill behavior such as material black mass.
- `hatching_strategy` selects optional character hatching.
- `budget_strategy` selects detail suppression and line importance policy.
- `temporal_strategy` selects temporal coherence policy.

The backend should read these through `NprPipelinePlan3d`, not through duplicated ad-hoc policy logic.

## Families And Brushes

The preset framework treats a style as a set of line families, not a single flat bundle of stroke multipliers.

- `line_families` decide which geometric candidates survive, how long strokes should become, how aggressively segments may join, and how much technical detail should remain.
- `brush_profiles` decide how a surviving family is actually drawn: tool class, width/alpha envelopes, taper, overshoot, path adherence, and directional nib behavior.
- `family preferences` let one preset split lines by intent inside the same geometric source, for example prefer one family for `technical_detail`, another for `ink_detail_material`, and another for `material_seam`.

This separation is intentional:

- family = editorial decision about what line exists
- brush = physical decision about how that line is inked

Four-point `width_curve` and `alpha_curve` already support envelopes such as "90% alpha at entry, full black in the middle, softer release at the tail". `angle_bias_degrees`, `angle_influence`, and `path_adherence_multiplier` let authored brushes behave more like actual nibs instead of pure geometric offsets.

`continuation_bias` and `breakup_bias` belong to line families rather than brushes:

- `continuation_bias` says how willing a family is to keep chaining candidate segments into one longer stroke.
- `breakup_bias` says how willing a family is to stop early and preserve shorter, more fragmentary marks.

This matters because "long clean contour" versus "short broken detail ink" is primarily a synthesis decision, not a nib decision.

Trait preferences are the bridge from raw mesh features to authored line roles:

- `technical_detail_preference` biases a family toward short technical candidates that still deserve ink.
- `ink_detail_material_preference` biases a family toward candidates touching authored ink-detail materials.
- `material_seam_preference` biases a family toward seam cuts and other authored surface boundaries.

That means a preset can say "feature lines on face or cloth-detail materials use a fine pen, but generic technical features stay suppressed" without adding a second renderer path.

## Diagnostics

Frame stats expose the active NPR pipeline plan as `world_3d_npr_pipeline_plan`. The dev console command `render.npr` prints this plan beside runtime counters. If a frame mixes multiple NPR plans, diagnostics report `mixed`.

## Preset Rules

- Presets should express intent through strategy fields and tool parameters, not backend-specific guesses.
- Akira-style presets should use `character_semantic`, `stable_stroked_paths`, `akira_ink`, and `stable_arc_length`.
- Toriyama-style clean manga presets should use `character_semantic`, `stable_stroked_paths`, `confident_manga_ink`, `character_readability`, and `stable_arc_length`.
- Search lines should stay disabled for `akira_ink` unless the preset intentionally wants exploratory sketch behavior.
- Sparse hatching should be paired with character-semantic candidates and camera response, so near/far detail can be budgeted coherently.

## Playground Preset Families

- `toriyama_1989_clean_ink` is the line-only pass: long confident contours, thin selected feature lines, no hatching, no search lines.
- `toriyama_1989_black_mass` adds material-driven black fills on top of the same line policy. It relies on mesh material roles, so presets keep material ID lists empty unless a model needs explicit overrides.
