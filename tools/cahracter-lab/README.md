# Character Lab

TypeScript SVG rig lab bundled to a classic browser script.

## Structure

- `src/` contains the TypeScript source split into app, rig, render, math, and export layers.
- `src/anatomy-rig.ts` defines the current anatomy tree as a `BodyPart` composite tree.
- `src/scene-graph.ts` evaluates `BodyPart` nodes into view-dependent render data and render passes.
- `src/styles.css` contains the authored styles copied into `dist/styles.css` by the build.
- `assets/source-rig.svg` is the canonical rig asset exported from the embedded source drawing.
- `index.html` is the root entry page and references built files from `dist/`.
- `dist/` contains the generated browser-ready bundle.

## Commands

- `npm install`
- `npm run check`
- `npm run build`
- `npm run dev`

## Notes

Open `index.html` after a build if you want `file://` usage. Use `Eksportuj widok SVG` to download the current generated frame.
Open `index-realism.html` if you want to preview the alternate, more proportional rig variant without replacing the base one.
