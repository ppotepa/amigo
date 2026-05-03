import React, { createContext, useContext, useEffect, useMemo, useState } from "react";
import { DEFAULT_FONT_ID, fontById, isFontId } from "./fontRegistry";
import type { FontId } from "./fontRegistry";
import { DEFAULT_THEME_ID, isThemeId } from "./themeRegistry";
import type { ThemeId } from "./themeTypes";
import { getThemeSettings, setThemeSettings } from "../api/editorApi";

const THEME_STORAGE_KEY = "amigo-editor.theme";
const FONT_STORAGE_KEY = "amigo-editor.font";

interface ThemeServiceValue {
  activeThemeId: ThemeId;
  previewThemeId: ThemeId | null;
  effectiveThemeId: ThemeId;
  activeFontId: FontId;
  previewFontId: FontId | null;
  effectiveFontId: FontId;
  setPreviewTheme: (themeId: ThemeId | null) => void;
  setPreviewFont: (fontId: FontId | null) => void;
  applyTheme: (themeId: ThemeId) => Promise<void>;
  applyFont: (fontId: FontId) => void;
  cancelPreview: () => void;
}

const ThemeServiceContext = createContext<ThemeServiceValue | null>(null);

function readInitialTheme(): ThemeId {
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  return isThemeId(stored) ? stored : DEFAULT_THEME_ID;
}

function readInitialFont(): FontId {
  const stored = window.localStorage.getItem(FONT_STORAGE_KEY);
  return isFontId(stored) ? stored : DEFAULT_FONT_ID;
}

export function ThemeServiceProvider({ children }: { children: React.ReactNode }) {
  const [activeThemeId, setActiveThemeId] = useState<ThemeId>(readInitialTheme);
  const [previewThemeId, setPreviewThemeId] = useState<ThemeId | null>(null);
  const [activeFontId, setActiveFontId] = useState<FontId>(readInitialFont);
  const [previewFontId, setPreviewFontId] = useState<FontId | null>(null);
  const effectiveThemeId = previewThemeId ?? activeThemeId;
  const effectiveFontId = previewFontId ?? activeFontId;

  useEffect(() => {
    document.documentElement.dataset.theme = effectiveThemeId;
  }, [effectiveThemeId]);

  useEffect(() => {
    document.documentElement.style.setProperty("--font-ui-active", fontById(effectiveFontId).cssValue);
  }, [effectiveFontId]);

  useEffect(() => {
    void (async () => {
      try {
        const settings = await getThemeSettings();
        if (isThemeId(settings.activeThemeId)) {
          setActiveThemeId(settings.activeThemeId);
          window.localStorage.setItem(THEME_STORAGE_KEY, settings.activeThemeId);
        }
      } catch {
        window.localStorage.setItem(THEME_STORAGE_KEY, activeThemeId);
      }
    })();
  }, [activeThemeId]);

  const value = useMemo<ThemeServiceValue>(
    () => ({
      activeThemeId,
      previewThemeId,
      effectiveThemeId,
      activeFontId,
      previewFontId,
      effectiveFontId,
      setPreviewTheme: setPreviewThemeId,
      setPreviewFont: setPreviewFontId,
      applyTheme: async (themeId) => {
        const previousThemeId = activeThemeId;
        setActiveThemeId(themeId);
        setPreviewThemeId(null);
        try {
          const settings = await setThemeSettings(themeId);
          if (!isThemeId(settings.activeThemeId)) {
            throw new Error("invalid theme returned by backend");
          }

          setActiveThemeId(settings.activeThemeId);
          window.localStorage.setItem(THEME_STORAGE_KEY, settings.activeThemeId);
        } catch (error) {
          setActiveThemeId(previousThemeId);
          setPreviewThemeId(null);
          window.localStorage.setItem(THEME_STORAGE_KEY, previousThemeId);
          document.documentElement.dataset.theme = previousThemeId;
          throw error;
        }
      },
      applyFont: (fontId) => {
        setActiveFontId(fontId);
        setPreviewFontId(null);
        window.localStorage.setItem(FONT_STORAGE_KEY, fontId);
      },
      cancelPreview: () => {
        setPreviewThemeId(null);
        setPreviewFontId(null);
      },
    }),
    [activeFontId, activeThemeId, effectiveFontId, effectiveThemeId, previewFontId, previewThemeId],
  );

  return <ThemeServiceContext.Provider value={value}>{children}</ThemeServiceContext.Provider>;
}

export function useThemeService(): ThemeServiceValue {
  const value = useContext(ThemeServiceContext);
  if (!value) {
    throw new Error("useThemeService must be used inside ThemeServiceProvider");
  }
  return value;
}
