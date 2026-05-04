# INK WARS Asset Structure

`ink-wars` uses descriptor-first assets. A runtime asset is a typed YAML descriptor under
`assets/<kind>/`. Binary/image/font/audio files are raw sources under `assets/raw/...`.

## Runtime Asset Descriptors

```text
assets/
  images/
    notebook-paper.image.yml
    notebook-paper-16x9.image.yml

  fonts/
    notebook-ink.font.yml

  tilesets/
    dirt.tileset.yml
    dirt-rules.tile-ruleset.yml

  tilemaps/
    dirt-test.tilemap.yml
```

Stable asset keys are derived from descriptor kind and id:

```text
ink-wars/images/notebook-paper-16x9
ink-wars/fonts/notebook-ink
ink-wars/tilesets/dirt
ink-wars/tilesets/dirt-rules
ink-wars/tilemaps/dirt-test
```

## Raw Source Files

Raw files are not managed runtime assets by themselves. They can be previewed, but they become
game/editor assets only after a typed descriptor references them.

```text
assets/raw/
  images/
    notebook-paper.jpg
    notebook-paper-16x9.jpg
    notebook-ink.png
    dirt.png
    dirt.upload.png
    dirt-source-layout.generated.png

  other/
    notebook-font-spritesheet.reference.yml
```

Scene YAML must not reference raw files directly. It references asset keys.

## Descriptor Rules

New descriptors use:

```text
<asset-id>.<asset-kind>.yml
```

Examples:

```text
notebook-paper.image.yml
notebook-ink.font.yml
dirt.tileset.yml
dirt-test.tilemap.yml
```

Old folders such as `textures/`, `fonts/`, `tilesets/` are no longer active asset roots for this
mod. They should stay empty or be removed after migration.

