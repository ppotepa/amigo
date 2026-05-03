# Amigo Editor Component System

## Problem

The main editor UI is no longer a single hardcoded layout. Startup, workspace docks, center tabs, settings, theme tools, diagnostics, event logs and future domain tools all need one shared model.

The editor should not treat UI as fixed regions like left sidebar or right inspector. It should treat visible UI as registered editor components.

## Core Rule

Everything visible in `amigo-editor` is an Editor Component.

An Editor Component is a typed, registered UI unit with:

- stable component ID,
- category,
- domain and optional subdomain,
- icon,
- placement rules,
- context requirements,
- capability requirements,
- metadata for docking, center tabs, floating panels or windows.

Docking/windowing is controlled by shell managers. React components do not move or dock themselves.

## Component Definition

A component definition describes the type of UI:

```txt
project.explorer exists
category: explorer
domain: project
default placement: left dock
context: editor session
```

Definition metadata is stable and belongs in the component registry.

## Component Instance

A component instance is one opened occurrence:

```txt
scene.preview for session editor-session-1 and scene main
yaml.editor for scenes/main/scene.yml
entity.inspector for selected entity
```

Instances are what layouts store. Layouts should not store React nodes.

## Categories

Initial categories:

- `workspace`
- `explorer`
- `inspector`
- `editor`
- `preview`
- `diagnostics`
- `tools`
- `settings`
- `system`
- `debug`

## Domains

Initial domains mirror the editor shell and engine-facing areas:

- `editor`
- `project`
- `modding`
- `scene`
- `assets`
- `scripting`
- `preview`
- `cache`
- `theme`
- `settings`
- `windowing`
- `diagnostics`
- `runtime`
- `rendering_2d`
- `motion_2d`
- `physics_2d`
- `particles_2d`
- `audio`
- `tilemap`
- `tileset`

## Placement

Components declare where they can live:

- `leftDock`
- `rightDock`
- `bottomDock`
- `centerTab`
- `floatingPanel`
- `window`
- `modal`

Examples:

- Project Explorer: `leftDock`, can float.
- Scene Preview: `centerTab`, can float or open in a window.
- Inspector: `rightDock`, can float.
- Event Log: `bottomDock`, can float or open in a window.
- Theme Controller: `window`.
- Settings: `window`.

## Capability Awareness

Components can require capabilities:

- Motion Inspector requires `motion_2d`.
- Particle Editor requires `particles_2d`.
- Physics Debugger requires `physics_2d`.
- Sprite Preview requires `rendering_2d`.

The component registry exposes this metadata, but the current MVP only uses it for filtering/availability. Future domain tools should use the same model.

## Current File Structure

```txt
src/editor-components/
├─ componentTypes.ts
├─ componentRegistry.tsx
├─ componentInstances.ts
├─ componentEvents.ts
├─ componentHost.tsx
├─ builtinComponents.tsx
└─ componentMenu.ts
```

## MVP Components

Dock/center components:

- `project.explorer`
- `assets.browser`
- `scene.hierarchy`
- `scene.preview`
- `entity.inspector`
- `diagnostics.problems`
- `events.log`
- `tasks.monitor`
- `cache.preview`

## Project Tab Source Of Truth

The left dock `Project` tab renders an engine-aware project structure tree, not a raw filesystem tree.

The backend owns the structure contract through `get_project_structure_tree(modId)`, returning:

- `EditorProjectStructureTreeDto`
- `EditorProjectStructureNodeDto`

The tree includes real filesystem state and expected editor/engine structure:

- root mod node,
- `Overview`,
- `mod.toml`,
- `scenes` with `scene.yml` / `scene.rhai`,
- `assets` categories,
- `scripts`,
- `packages`,
- `Capabilities`,
- `Dependencies`,
- `Diagnostics`.

Missing expected folders are represented as `ghost` nodes with `exists: false`, `ghost: true`, and status `missing`. Empty existing folders use `exists: true`, `empty: true`, and status `empty`.

The frontend renders the DTO and emits node activation/context actions. It should not be the primary place where expected mod structure is defined.

Window-only components:

- `theme.controller`
- `settings.global`
- `settings.mod`
- `cache.manager`

Future domain components:

- `scripting.rhai-editor`
- `rendering_2d.sprite-preview`
- `motion_2d.inspector`
- `physics_2d.debugger`
- `particles_2d.emitter-editor`

## Implementation Order

1. Add component definitions and registry.
2. Generate dock plugin metadata from registered components.
3. Add component instances and instance IDs.
4. Add ComponentHost.
5. Move workspace layout state from plugin IDs to component instance IDs.
6. Add generated Window/Panels menus.
7. Persist layout state.
8. Add external plugin loading only after the built-in registry stabilizes.

## Design Decision

The component system is an `amigo-editor` shell subsystem. It is not part of the engine. If the component protocol stabilizes and needs sharing with external tools, it can later move to an `amigo-editor-protocol` crate/package.
