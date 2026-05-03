import type { ThemeDefinition, ThemeId } from "./themeTypes";

export const THEMES: ThemeDefinition[] = [
  {
    id: "amigo-dark-navy",
    name: "Dark Navy",
    description: "Deep blue-black theme for long editor sessions.",
    mode: "dark",
    accent: "#3b82f6",
  },
  {
    id: "amigo-light-paper",
    name: "Light Paper",
    description: "Bright high-contrast theme for daylight work.",
    mode: "light",
    accent: "#2563eb",
  },
];

export const DEFAULT_THEME_ID: ThemeId = "amigo-dark-navy";

export function isThemeId(value: string | null): value is ThemeId {
  return value === "amigo-dark-navy" || value === "amigo-light-paper";
}
