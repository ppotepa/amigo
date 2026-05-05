import type { ThemeId } from "./themeTypes";

const TOKEN_GROUPS: Array<[string, string[]]> = [
  ["App", ["--color-app-bg", "--color-app-bg-elevated"]],
  ["Surface", ["--color-surface", "--color-surface-raised", "--color-surface-hover"]],
  ["Text", ["--color-text-primary", "--color-text-secondary", "--color-text-muted"]],
  ["Status", ["--color-success", "--color-warning", "--color-danger", "--color-info"]],
  ["Accent", ["--color-accent", "--color-accent-hover", "--color-accent-soft"]],
  ["Domain", [
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
  ]],
  ["Asset", [
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
  ]],
  ["Shadows", ["--shadow-panel", "--shadow-popover", "--shadow-dock", "--shadow-dock-inset", "--shadow-toolbar", "--shadow-preview-frame"]],
];

export function ThemeTokenInspector({ themeId }: { themeId: ThemeId }) {
  return (
    <aside className="theme-token-panel" data-theme={themeId}>
      <h3>Theme Tokens</h3>

      {TOKEN_GROUPS.map(([group, tokens]) => (
        <section key={group} className="token-group">
          <h4>{group}</h4>
          {tokens.map((token) => (
            <div key={token} className="token-row">
              <span
                className="token-swatch"
                style={
                  token.startsWith("--shadow")
                    ? { background: "var(--color-surface)", boxShadow: `var(${token})` }
                    : { background: `var(${token})` }
                }
              />
              <code>{token}</code>
            </div>
          ))}
        </section>
      ))}
    </aside>
  );
}
