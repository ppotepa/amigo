# Amigo Editor Visual Refresh

## 1. Design Direction: Fresh Minimal UI v1

The editor UI should feel like a focused tool surface: flatter panels, subtler shadows, restrained accents, and clear interaction states. Visual polish must come from shared design tokens and component classes, not one-off CSS in individual views.

## 2. Radius Scale

Use a tighter radius scale:

```txt
xs: 3px
sm: 6px
md: 9px
lg: 12px
xl: 14px
```

## 3. Typography

UI font stack:

```txt
Geist, Inter, system sans
```

Mono font stack:

```txt
JetBrains Mono, Geist Mono, Consolas
```

## 4. Shadows / Elevation

Panel shadows should be subtle. Popovers and modals can use stronger elevation, but avoid heavy dashboard-style shadowing.

## 5. Interaction Tokens

Hover, active, focus and disabled states must be backed by global tokens:

```txt
interactive bg
interactive border
interactive ring
motion fast / normal
opacity muted / disabled
```

## 6. Button System

Use shared variants:

```txt
primary-button
secondary-button
ghost-button
tool-button
```

Primary is reserved for the main action. Utility and toolbar actions should remain visually quiet.

## 7. Badge System v2

Status display uses compact text badges with a small status dot:

```txt
badge-valid
badge-warning
badge-error
badge-info
badge-muted
```

Avoid status dots without labels when the state matters.

## 8. Panel System

Panels are flat surfaces with one border and a subtle shadow. Avoid nested heavy cards. Inspector sections can use simple separators or low-contrast surfaces.

## 9. Row / List Item System

Rows are flat by default. Hover adds surface color and border. Selected rows use accent soft background and accent border.

## 10. Theme Palette Update

Dark Navy remains the default, but it should be less glow-heavy and more legible. Light Paper should stay clean and high contrast.

## 11. Theme Preview Sample

The Theme Controller preview should show representative UI pieces: buttons, project rows, badges, diagnostics and panel surfaces.

## 12. Migration Checklist

```txt
1. Update tokens.css.
2. Update themes.css.
3. Add shared interactive/button/badge classes.
4. Replace status dots with badges.
5. Flatten panel and row styles.
6. Refresh ThemePreviewPanel.
7. Refresh Settings rows/project index.
8. Refresh EditorWorkspace.
9. Run CSS scan.
10. Run build/check.
```

## 13. Acceptance Criteria

```txt
- less rounded UI
- subtle shadows
- consistent hover/active/focus states
- badge v2 status labels
- no random hardcoded colors outside theme definitions
- Startup, Settings, Theme Controller and Editor Workspace feel coherent
- build/check pass
```
