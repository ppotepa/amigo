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

## Diagnostics

Frame stats expose the active NPR pipeline plan as `world_3d_npr_pipeline_plan`. The dev console command `render.npr` prints this plan beside runtime counters. If a frame mixes multiple NPR plans, diagnostics report `mixed`.

## Preset Rules

- Presets should express intent through strategy fields and tool parameters, not backend-specific guesses.
- Akira-style presets should use `character_semantic`, `stable_stroked_paths`, `akira_ink`, and `stable_arc_length`.
- Search lines should stay disabled for `akira_ink` unless the preset intentionally wants exploratory sketch behavior.
- Sparse hatching should be paired with character-semantic candidates and camera response, so near/far detail can be budgeted coherently.
