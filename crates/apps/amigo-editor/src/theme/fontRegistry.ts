export type FontId = "source-sans-3" | "geist-sans" | "segoe-ui" | "system-ui";

export interface FontDefinition {
  id: FontId;
  name: string;
  description: string;
  cssValue: string;
}

export const DEFAULT_FONT_ID: FontId = "source-sans-3";

export const FONTS: FontDefinition[] = [
  {
    id: "source-sans-3",
    name: "Source Sans 3",
    description: "Lighter editorial UI feel for dense panels.",
    cssValue: '"Source Sans 3", "Geist Sans", ui-sans-serif, system-ui, sans-serif',
  },
  {
    id: "geist-sans",
    name: "Geist Sans",
    description: "Crisp technical product UI.",
    cssValue: '"Geist Sans", "Inter", ui-sans-serif, system-ui, sans-serif',
  },
  {
    id: "segoe-ui",
    name: "Segoe UI",
    description: "Native Windows UI baseline.",
    cssValue: '"Segoe UI", ui-sans-serif, system-ui, sans-serif',
  },
  {
    id: "system-ui",
    name: "System UI",
    description: "Platform default sans stack.",
    cssValue: 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
];

export function isFontId(value: string | null): value is FontId {
  return value === "source-sans-3" || value === "geist-sans" || value === "segoe-ui" || value === "system-ui";
}

export function fontById(fontId: FontId): FontDefinition {
  return FONTS.find((font) => font.id === fontId) ?? FONTS[0];
}
