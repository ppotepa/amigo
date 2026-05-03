# Amigo Editor Theme System

## Goal

`amigo-editor` has a first-class theme system. Themes are not one-off CSS files; they are editor services that provide semantic design tokens to every screen.

Initial themes:

```txt
amigo-dark-navy
amigo-light-paper
amigo-mexico
amigo-mexico-dark
```

## Rules

Production components must not define palette colors directly. Components consume semantic CSS variables:

```css
background: var(--color-surface);
color: var(--color-text-primary);
border-color: var(--color-border);
```

Actual values live in `src/styles/themes.css`.

## Frontend Shape

```txt
src/theme/
├─ themeTypes.ts
├─ themeRegistry.ts
├─ themeService.tsx
├─ themeEvents.ts
├─ ThemeButton.tsx
├─ ThemeControllerDialog.tsx
├─ ThemePreviewPanel.tsx
└─ ThemeTokenInspector.tsx
```

`ThemeServiceProvider` owns:

```txt
activeThemeId
previewThemeId
effectiveThemeId
setPreviewTheme()
applyTheme()
cancelPreview()
```

The provider writes `data-theme` on `document.documentElement`.

## Persistence

Current implementation persists the selected theme in `localStorage`:

```txt
amigo-editor.theme
```

Backend settings commands exist as a stubbed persistence surface:

```txt
get_theme_settings()
set_theme_settings(theme_id)
```

Later they should read/write editor settings in AppConfig/AppData.

## UX

Startup Dialog header exposes a `Theme` button. It opens `ThemeControllerDialog`.

Selection behavior:

```txt
click theme -> temporary full-app preview
Apply Theme -> persist active theme
Cancel      -> restore previous active theme
```

The dialog previews real editor UI language: panels, tree rows, toolbar buttons, badges, diagnostics, and a scene preview area.
