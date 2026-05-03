# Tiles Editor Package

This package collects the current design and implementation notes for the first real Amigo Editor workspace editor: **Sheet Editor Core + TileSet Editor mode**.

It includes:

- `docs/tiles-editor-overview.md` — product/architecture overview.
- `docs/tiles-editor-functional-spec.md` — full feature set.
- `docs/tiles-editor-implementation-plan.md` — step-by-step implementation plan.
- `docs/engine-required-changes.md` — required engine/runtime changes.
- `docs/backend-contract.md` — backend DTOs and commands.
- `docs/frontend-components.md` — frontend component structure.
- `specs/example-dirt.tileset.yml` — proposed normalized tileset schema.
- `specs/example-dirt.tilemap.yml` — future tilemap schema stub.
- `mockups/tileset-editor-mockup.html` — standalone visual mockup of the editor.
- `mockups/sheet-canvas-layout.txt` — ASCII layout.

Primary target for the first iteration:

```txt
Ink Wars dirt tileset
current: dirt.semantic.yml
future:  dirt.tileset.yml
```

Core decision:

```txt
Build Sheet Editor Core first, then use it for TileSet Editor and later SpriteSheet Editor / TileMap Editor.
```
