# Mod asset structure

This repository uses a shared contract for mod assets. The goal is:
- runtime assets in named responsibility folders,
- scene-local assets only when they are truly local,
- semantic meaning stored in YAML metadata rather than extra folder depth,
- non-runtime authoring files isolated in `sources/`.

## Standard mod layout

```text
mods/<mod-id>/
  mod.toml

  docs/
  sources/
  scenes/

  scripts/
  data/

  fonts/
  textures/
  tilesets/
  audio/
  materials/
  meshes/
  presets/
  particles/
```

### Scene split convention

```text
scenes/
  shared/        # common scene fragments (optional)
  menu/
  match/
  lab/
```

Use `scenes/<scene-id>/` for per-scene entities only.

## Runtime shared assets

Folders listed below are directly consumable by scene/runtime keys:

```text
fonts/
textures/
tilesets/
audio/
materials/
meshes/
presets/
particles/
```

Examples:

```text
<mod-id>/fonts/<asset>
<mod-id>/textures/<asset>
<mod-id>/tilesets/<asset>
<mod-id>/audio/<asset>
<mod-id>/particles/<asset>
```

Asset category should be controlled by metadata fields such as `group`, `tags`, `role`, `style`.

## Scene-local assets

Use scene-local directories only when an asset truly belongs only to one scene:

```text
scenes/<scene-id>/assets/
scenes/<scene-id>/data/
```

If an item is reused by more than one scene, promote it to a runtime folder.

## Source assets

`sources/` is **not runtime**. It stores intermediary/reference data:

```text
sources/
  # raw source references
  # edited source material
  # generated temporary/debug outputs
```

Scenes must not reference `sources/` directly.

## Data

`data/` is optional and holds shared logical configuration not tied to a single scene:

```text
data/
```

Store gameplay/domain semantics there, but keep data names stable.

## Docs

`docs/` stores human documentation and task-level design notes.

## Migration rule

Existing mods can migrate gradually:

- keep runtime paths stable;
- move non-runtime files to `sources/`;
- promote shared scene-only content to shared/runtime folders;
- keep folder structure by responsibility, not by art narrative.

## Compatibility

- Missing optional folders are valid (`scripts/`, `audio/`, `particles/` etc. may be absent).
- This is a repo-wide contract, not a strict one-shot migration order.
