# char-3d

Static NPR/vector projection playground for OBJ and FBX sources.

## Run

```powershell
npm install
npm run dev
```

Open the URL printed by Vite and use `strokes.html`.

## Checks

```powershell
npm run check
npm run build
npm run perf
```

`strokes.html` must be served over HTTP because OBJ/FBX assets are loaded with `fetch`.

## Runtime Notes

- The main render pipeline caches the last projected `RenderFrame`; paint/color/debug-only edits redraw from the cached frame instead of recomputing projection, depth, contours, and marks.
- Paint is rendered as projected organic regions (`watercolor`, `gouache`, `comicCel`, `inkWash`) generated from visible surface tone instead of final per-triangle color fills.
- The FBX adapter caches topology once and updates animated vertex positions per sampled frame.
- Dirty scopes (`mesh`, `projection`, `visibility`, `npr`, `paint`, `display`) are tracked so UI edits can avoid unnecessary pipeline work.
- The status panel reports per-stage timings and whether the last frame was a cache hit or miss.
