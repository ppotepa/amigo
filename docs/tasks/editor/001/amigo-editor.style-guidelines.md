# Amigo Editor Style Guidelines

## Visual Direction

`amigo-editor` should feel like a modern desktop tool for game assets: compact, readable, technical, and calm. The first visual direction is dark blue.

Primary goals:

- clear hierarchy,
- high readability,
- low visual noise,
- editor/tool feeling rather than marketing UI,
- enough contrast for long sessions.

## Theme Name

```txt
Amigo Dark Navy
```

The implemented editor has two built-in themes:

```txt
amigo-dark-navy
amigo-light-paper
```

See `amigo-editor.theme-system.md` for the theme controller and token contract.

## Color Tokens

```txt
--bg-app:              #07111f
--bg-window:           #0a1626
--bg-surface:          #0d1b2e
--bg-surface-raised:   #13243a
--bg-surface-hover:    #182d47
--bg-input:            #081524

--border-subtle:       #1d314a
--border-strong:       #2a4668

--text-primary:        #e6edf7
--text-secondary:      #b5c3d6
--text-muted:          #7f93ad
--text-disabled:       #4f6278

--accent:              #3b82f6
--accent-hover:        #60a5fa
--accent-soft:         #102b55
--accent-border:       #1d4ed8

--success:             #22c55e
--success-soft:        #0f3323
--warning:             #f59e0b
--warning-soft:        #3a2808
--error:               #ef4444
--error-soft:          #3a1116
--info:                #38bdf8
--info-soft:           #0b2d3b
```

## Surface Hierarchy

Use a small number of surfaces consistently:

```txt
app background      -> --bg-app
main dialog/window  -> --bg-window
panel               -> --bg-surface
raised card         -> --bg-surface-raised
hover/selected      -> --bg-surface-hover
```

Avoid random blues. Every new component should reuse an existing token first.

## Typography

Recommended UI font stack:

```css
font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
```

Use compact type sizes:

```txt
caption:       11px
small:         12px
body:          13px
body-large:    14px
section title: 13px / uppercase / letter-spaced
page title:    18px-20px
```

## Spacing

Use compact desktop spacing:

```txt
4px   tiny gap
8px   standard inner gap
12px  panel padding
16px  section padding
20px  outer dialog padding
```

Avoid large landing-page spacing.

## Radius

```txt
small controls:  6px
cards/panels:    10px
dialog/window:   16px
```

## Borders and Shadows

Borders should carry most of the structure. Shadows should be subtle.

```txt
panel border: 1px solid --border-subtle
active border: 1px solid --border-strong
focus border: 1px solid --accent
```

Avoid heavy shadows. This should feel like an editor, not a floating marketing card.

## Status Colors

Use status color only for meaning:

```txt
success -> valid mod, healthy scene, ready cache
warning -> missing optional asset, outdated cache, schema warning
error   -> invalid file, missing required field, failed preview
info    -> scanning, generated preview, metadata note
```


## Icon System

The startup dialog is the first reference screen for the whole editor, so icons should be treated as part of the design system, not as decoration.

Primary production icon set:

```txt
Lucide React
```

Mockup rule:

```txt
Standalone HTML mockups may use inline SVG icons that mimic the final Lucide line style.
```

Icon style:

```txt
stroke: 1.75px-2px
style: outline / line icons
corners: rounded
size: 14px-18px in dense UI
button icons: 13px-15px
panel title icons: 15px-16px
```

Icon usage rules:

- every tree folder/file row should have an icon,
- every expandable tree group should use a chevron,
- icon meaning must stay stable across the app,
- icons should inherit text/status color using `currentColor`,
- avoid mixing filled, emoji, bitmap, and line icons,
- use icons to support labels, not replace labels in important actions.

Recommended base icon mapping:

```txt
workspace/root      -> folder-open
mod/project         -> folder
recent item         -> clock
scene               -> film / panels-top-left
image/sprite        -> image
spritesheet/atlas   -> grid-2x2 / layout-grid
tileset             -> grid-3x3
tilemap             -> map
audio               -> audio-lines / waveform
script              -> file-code
yaml/metadata       -> file-text
validation valid    -> shield-check
warning             -> triangle-alert
error               -> circle-x
refresh             -> refresh-cw
settings            -> settings
open/browse         -> folder-open / folder-plus
```

The editor contract layer may provide semantic icon keys for known folders and asset kinds. The frontend maps those keys to the visual icon library.

Example:

```txt
AssetKind::Scene      -> icon: scene
AssetKind::Audio      -> icon: audio
FolderKind::Scenes    -> icon: folder-scenes
FolderKind::Scripts   -> icon: folder-code
```

## First Dialog as Visual Precedent

The startup dialog should define reusable patterns for later screens:

- panel headers,
- icon buttons,
- tree rows,
- validation badges,
- metadata lists,
- preview cards,
- primary/secondary actions,
- compact dark-blue surface hierarchy.

Future mockups should reuse these patterns unless there is a clear reason to add a new component pattern.

## Mod Tree Guidelines

The mod tree should look like a classic desktop tree:

- chevrons for expandable nodes,
- folder/file icons,
- small validation badges,
- selected row highlight,
- muted metadata such as version or scene count,
- no oversized cards inside the tree.

## Startup Dialog Guidelines

Note: `startup-dialog.html` and `styles.css` in this task folder are the original v1 static mockup references. The implemented startup dialog now follows the v3 interactive preview launcher layout below.

Target size:

```txt
1340x880
```

Main layout:

```txt
left   -> available mods with search/filter
center -> interactive scene preview workspace
right  -> scrollable mod/scene inspector
bottom -> event/status log and actions
```

The dialog should communicate three things quickly:

1. Which mod is selected?
2. What does its selected scene look like?
3. Is the mod valid and what does it contain?

Recommended fixed-window layout:

```txt
left:   300px  Available Mods
center: fluid  Scene Preview Workspace
right:  340px  Mod Inspector
gap:     12px
```

The startup dialog should be fixed size and non-maximizable. The UI should fit without scrollbars.

## Scene Preview Cards

Scene preview is the primary center workspace. It should show:

- one large selected scene preview area,
- scene title and path,
- status: ready, missing, failed, queued, or rendering,
- 5 FPS badge,
- a compact scene strip below.

When preview fails, show a clear fallback state instead of an empty box.

Do not show filler thumbnails or decorative fake preview art. Use a real cache/render result or an explicit status/error state.

The main scene preview must always fit the available preview canvas. The launcher should not expose a 1:1 mode on this screen; it is for recognizing the mod before opening it, not inspecting pixels.

Metadata/SVG previews are no longer allowed for the implemented startup dialog. Scene preview must come from the engine offscreen render path and return either cached/generated slideshow frames or an explicit `missing`/`failed` state with diagnostics.

Startup slideshow rules:

```txt
source        -> ScenePreviewHost / offscreen WGPU render
cache         -> target/amigo-editor/cache/projects/<project-cache-id>/previews/scenes/<scene>/<hash>/
frames        -> frame_000.png ... frame_014.png
playback      -> 5 FPS
duration      -> 3 seconds
thumbnail     -> first real frame
failure state -> no placeholder; show diagnostics
```

## Mod Details Panel

The right panel should behave like an inspector. It should include:

```txt
Mod Details
- Name
- Mod ID
- Version
- Authors
- Root
- Description

Content Summary
- scenes
- scene YAML
- scripts
- textures
- spritesheets
- audio
- tilemaps
- tilesets
- packages
- unknown files

Engine
- capabilities
- dependencies
- contract/version

Diagnostics
- errors
- warnings
```

## Buttons

Primary action:

```txt
Open Mod
```

Secondary actions:

```txt
Browse...
Refresh Preview
Cancel
```

Primary button should use accent blue. Secondary buttons should stay neutral.

## Accessibility Baseline

- text must remain readable on dark surfaces,
- selected states must not rely only on color,
- focus states must be visible,
- disabled states must be clearly muted,
- badges must have enough contrast.

## Design Rule

When in doubt, prefer:

```txt
more readable > more decorative
compact > spacious
clear borders > heavy shadows
one accent color > many accent colors
```
