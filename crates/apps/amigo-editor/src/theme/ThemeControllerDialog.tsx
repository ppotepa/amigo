import { Check, Moon, Paintbrush, Sun, Type } from "lucide-react";
import { DEFAULT_FONT_ID, FONTS } from "./fontRegistry";
import { DEFAULT_THEME_ID, THEMES } from "./themeRegistry";
import { useThemeService } from "./themeService";
import { ThemePreviewPanel } from "./ThemePreviewPanel";
import { ThemeTokenInspector } from "./ThemeTokenInspector";
import { useEditorStore } from "../app/editorStore";
import { useEffect } from "react";

export function ThemeControllerContent({ onClose }: { onClose?: () => void }) {
  const {
    activeFontId,
    activeThemeId,
    effectiveFontId,
    effectiveThemeId,
    setPreviewFont,
    setPreviewTheme,
    applyFont,
    applyTheme,
    cancelPreview,
  } = useThemeService();
  const { recordEvent } = useEditorStore();

  useEffect(() => {
    recordEvent({ type: "ThemeControllerOpened" });
  }, [recordEvent]);

  async function handleApply() {
    recordEvent({ type: "ThemeApplyRequested", themeId: effectiveThemeId });
    try {
      await applyTheme(effectiveThemeId);
      applyFont(effectiveFontId);
      recordEvent({ type: "ThemeApplied", themeId: effectiveThemeId });
      onClose?.();
    } catch (error) {
      recordEvent({
        type: "ThemeApplyFailed",
        themeId: effectiveThemeId,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }

  function handleCancel() {
    recordEvent({ type: "ThemePreviewCancelled" });
    cancelPreview();
    onClose?.();
  }

  return (
    <section className="theme-dialog" role="dialog" aria-modal="true" aria-labelledby="theme-dialog-title">
      <header className="theme-dialog-header">
        <div>
          <h2 id="theme-dialog-title">
            <Paintbrush size={18} />
            Theme Controller
          </h2>
          <p>Preview and apply the visual theme for Amigo Editor.</p>
        </div>

        <span className="pill">Current: {activeThemeId}</span>
      </header>

      <main className="theme-dialog-grid">
        <aside className="theme-list-panel">
          <h3>Available Themes</h3>
          {THEMES.map((theme) => (
            <button
              key={theme.id}
              type="button"
              className={`theme-list-item ${effectiveThemeId === theme.id ? "selected" : ""}`}
              onClick={() => {
                setPreviewTheme(theme.id);
                recordEvent({ type: "ThemePreviewStarted", themeId: theme.id });
              }}
            >
              {theme.mode === "dark" ? <Moon size={17} /> : <Sun size={17} />}
              <span>
                <strong>{theme.name}</strong>
                <small>{theme.description}</small>
              </span>
              {activeThemeId === theme.id ? <Check size={16} /> : null}
            </button>
          ))}

          <h3 className="font-list-title">UI Font</h3>
          <div className="font-list">
            {FONTS.map((font) => (
              <button
                key={font.id}
                type="button"
                className={`font-list-item ${effectiveFontId === font.id ? "selected" : ""}`}
                style={{ fontFamily: font.cssValue }}
                onClick={() => setPreviewFont(font.id)}
              >
                <Type size={15} />
                <span>
                  <strong>{font.name}</strong>
                  <small>{font.description}</small>
                </span>
                {activeFontId === font.id ? <Check size={15} /> : null}
              </button>
            ))}
          </div>
        </aside>

        <ThemePreviewPanel themeId={effectiveThemeId} />
        <ThemeTokenInspector themeId={effectiveThemeId} />
      </main>

      <footer className="theme-dialog-footer">
        <button
          className="button button-ghost"
          type="button"
          onClick={() => {
            setPreviewTheme(DEFAULT_THEME_ID);
            setPreviewFont(DEFAULT_FONT_ID);
          }}
        >
          Reset Preview
        </button>

          <div className="footer-actions">
            <button className="button button-ghost" type="button" onClick={handleCancel}>
              Cancel
            </button>
            <button className="button button-primary" type="button" onClick={handleApply}>
              Apply Theme
            </button>
          </div>
        </footer>
    </section>
  );
}

export function ThemeControllerDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  if (!open) {
    return null;
  }

  return (
    <div className="modal-backdrop">
      <ThemeControllerContent onClose={onClose} />
    </div>
  );
}
