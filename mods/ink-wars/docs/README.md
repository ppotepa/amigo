# INK WARS

Current prototype path:

```text
mods/ink-wars
```

Target mod identity:

```text
title: INK WARS
target id: ink-wars
visual target: blue pen sketch on school notebook paper
```

The canonical mod id and folder are now `ink-wars`. Game-specific scenes, labs, assets and scripts should live in this mod.

## Visual direction

Reference mockup:

```text
docs/mockups/ink-wars.png
docs/asset-structure.md
```

Notebook font metadata:

```text
assets/fonts/notebook-ink.font.yml
assets/raw/other/notebook-font-spritesheet.reference.yml
```

Expected source image path:

```text
assets/raw/images/notebook-ink.png
```

The game should read as a playable notebook drawing:

- paper background with horizontal notebook lines;
- left-side margin and spiral/holes;
- blue pen contours;
- hand-drawn split-screen divider;
- caves and terrain filled with hatching/crosshatching;
- crater edges redrawn after explosions;
- pickups as doodled crates/medkits/ammo boxes;
- players as small capsule/doodle soldiers with clear P1/P2 indicators;
- HUD as hand-drawn panels, bars and weapon icons.

## Notebook font

The supplied glyph sheet is treated as a bitmap spritesheet font:

- three horizontal variants;
- fixed-grid estimate: 9 columns by 10 rows per variant;
- estimated cell size: 74x66 pixels;
- default ink color: `#123D9AFF`;
- paper/white background should be extracted as transparent alpha;
- variant selection must be deterministic by UI node id and glyph index.

Current renderer state: UI text supports `font-2d` assets with `format: bitmap-spritesheet`, loads the raw source from `assets/raw/images/notebook-ink.png` through `assets/fonts/notebook-ink.font.yml`, extracts blue ink into alpha, and draws UI text as bitmap glyph quads. The atlas uses three horizontal frames and animates them at the configured `animation.fps`.

## Engine direction

The main new renderer is `ink_render_2d`.

Terrain gameplay remains data/simulation driven:

```text
mask -> damage brush -> dirty chunks -> collision/raycast queries
```

Terrain visuals are derived from the current mask:

```text
mask -> marching squares -> contour loops -> surface classification -> visual data -> ink commands
```

Do not render the notebook style as one static PNG. Static artwork can be used as reference or temporary placeholder only.

## System boundaries

`destructible_terrain_2d` owns:

- terrain mask;
- procedural generation;
- damage brushes;
- dirty regions/chunks;
- terrain collision/raycast;
- contour extraction;
- surface classification;
- visual extraction data.

`ink_render_2d` owns:

- paper rendering;
- notebook lines/margin/spiral;
- jittered strokes;
- hatch/crosshatch;
- doodle primitives;
- ink particle bridge;
- ink UI skin;
- terrain visual data to ink commands.

`viewport_2d` owns:

- split screen;
- multiple cameras;
- per-viewport HUD roots;
- camera follow and shake.

Pickup behavior should be entity-based. A pickup is an entity with collect behavior, active/inactive state, visual style and optional respawn. The mod should not depend on hardcoded `on_collision` game rules in the engine.

## Current gameplay milestone

The immediate playable milestone may stay single-player:

- one placeholder capsule player;
- stable terrain collision;
- camera follow;
- ink-styled HUD;
- first weapon/action feedback.

The full task is still a local duel:

- two local players;
- vertical split screen;
- per-player HUD;
- destructible terrain;
- 8 weapons;
- pickups;
- result screen.

## Task docs

Full implementation plan:

```text
docs/tasks/017/017.summary.md
docs/tasks/017/017.impl.md
```

