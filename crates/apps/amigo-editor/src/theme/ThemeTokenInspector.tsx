import type { ThemeId } from "./themeTypes";

const TOKEN_GROUPS: Array<[string, string[]]> = [
  ["App", ["--color-app-bg", "--color-app-bg-elevated", "--app-shell-bg"]],
  ["Surface", ["--color-surface", "--color-surface-raised", "--color-surface-hover"]],
  ["Text", ["--color-text-primary", "--color-text-secondary", "--color-text-muted"]],
  ["Status", ["--color-success", "--color-warning", "--color-danger", "--color-info"]],
  ["Accent", ["--color-accent", "--color-accent-hover", "--color-accent-soft", "--color-accent-secondary"]],
  [
    "Theme Chrome",
    ["--theme-app-bg", "--theme-chrome-bg", "--theme-chrome-border", "--theme-chrome-shadow"],
  ],
  [
    "Theme Panels",
    [
      "--theme-panel-bg",
      "--theme-panel-bg-raised",
      "--theme-panel-border",
      "--theme-panel-border-strong",
      "--theme-panel-shadow",
    ],
  ],
  [
    "Theme Controls",
    [
      "--theme-button-bg",
      "--theme-button-bg-hover",
      "--theme-button-border",
      "--theme-button-border-hover",
      "--theme-button-shadow",
      "--theme-button-shadow-hover",
      "--theme-primary-button-bg",
      "--theme-primary-button-border",
      "--theme-primary-button-shadow",
    ],
  ],
  [
    "Theme Selection",
    [
      "--theme-selection-bg",
      "--theme-selection-border",
      "--theme-selection-shadow",
      "--theme-active-tab-bg",
      "--theme-active-underline",
    ],
  ],
  ["Theme Inputs", ["--theme-input-bg", "--theme-input-border", "--theme-input-shadow", "--interactive-ring"]],
  [
    "Domain",
    [
      "--domain-editor-color",
      "--domain-project-color",
      "--domain-scene-color",
      "--domain-assets-color",
      "--domain-scripting-color",
      "--domain-preview-color",
      "--domain-cache-color",
      "--domain-theme-color",
      "--domain-diagnostics-color",
      "--domain-runtime-color",
      "--domain-rendering-2d-color",
      "--domain-physics-2d-color",
      "--domain-audio-color",
      "--domain-tilemap-color",
      "--domain-tileset-color",
    ],
  ],
  [
    "Asset",
    [
      "--asset-image-color",
      "--asset-sprite-color",
      "--asset-tileset-color",
      "--asset-tilemap-color",
      "--asset-audio-color",
      "--asset-font-color",
      "--asset-scene-color",
      "--asset-script-color",
      "--asset-raw-image-color",
      "--asset-generic-color",
    ],
  ],
  [
    "Shadows",
    [
      "--shadow-panel",
      "--shadow-popover",
      "--shadow-dock",
      "--shadow-dock-inset",
      "--shadow-toolbar",
      "--shadow-preview-frame",
      "--theme-glow-soft",
      "--theme-glow-strong",
      "--theme-preview-frame",
    ],
  ],
];

export function ThemeTokenInspector({ themeId }: { themeId: ThemeId }) {
  return (
    <aside className="theme-token-panel" data-theme={themeId}>
      <h3>Theme Tokens</h3>

      {TOKEN_GROUPS.map(([group, tokens]) => (
        <section key={group} className="token-group">
          <h4>{group}</h4>
          {tokens.map((token) => {
            const isEffectToken =
              token.includes("shadow") || token.includes("glow") || token.includes("ring");
            return (
              <div key={token} className="token-row">
                <span
                  className="token-swatch"
                  style={
                    isEffectToken
                      ? { background: "var(--color-surface)", boxShadow: `var(${token})` }
                      : { background: `var(${token})` }
                  }
                />
                <code>{token}</code>
              </div>
            );
          })}
        </section>
      ))}
    </aside>
  );
}
