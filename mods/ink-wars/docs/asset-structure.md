# INK WARS asset structure

`ink-wars` follows the repository-wide mod asset contract.

## Runtime assets

Runtime assets are files that scenes can reference by asset key.

```text
fonts/
  notebook-ink.yml
  notebook-ink.png
  debug-ui.toml

textures/
  notebook-paper.yml
  notebook-paper.jpg
  notebook-paper-16x9.yml
  notebook-paper-16x9.jpg

tilesets/
  earth.yml
  earth.png
  earth-rules.yml
```

Asset keys intentionally stay compatible with the current resolver:

```text
ink-wars/fonts/notebook-ink
ink-wars/textures/notebook-paper-16x9
ink-wars/tilesets/earth
ink-wars/tilesets/earth-rules
```

## Source assets

Source uploads and generated intermediate files are kept out of runtime folders:

```text
sources/
  earth.upload.png
  earth-source-layout.png
  notebook-font-spritesheet.yml
```

Scene YAML must not reference these files directly.

Semantic tile metadata that is not renderer-facing can stay in the same `tilesets/` namespace:

```text
tilesets/
  earth.semantic.yml
```

Current engine still uses inline `TileMap2D.grid` in `scene.yml`. When external tilemap sources are supported,
`scenes/match/<name>.yml` should replace large inline maps.

## Shared scene ownership (repo pattern)

`ink-wars` follows the shared scene convention for repo-wide consistency:

```text
scenes/
  shared/        # optional shared entities/groups used by more than one scene
  menu/
  match/
  terrain-lab/
  split-screen-lab/
  weapon-lab/
```

If a scene component is only used in one scene, keep it inside that scene entity set.

## Source files

`sources/` is for non-runtime artifacts only. For runtime use, keep files under:

```text
fonts/
textures/
tilesets/
```
