import type { ThemeDefinition, ThemeId } from "./themeTypes";

export const THEMES: ThemeDefinition[] = [
  {
    id: "mexico-at-night",
    name: "Mexico at Night",
    description: "Moonlit desert workspace with navy surfaces, agave accents, and amber highlights.",
    mode: "dark",
    accent: "#3A6D73",
  },
  {
    id: "mexico-sand",
    name: "Mexico Sand",
    description: "Warm sandy workspace with cream surfaces and soft agave / terracotta accents.",
    mode: "light",
    accent: "#6FA487",
  },
  {
    id: "amigo-light-paper",
    name: "Light Paper",
    description: "Bright high-contrast theme for daylight work.",
    mode: "light",
    accent: "#2563eb",
  },
];

export const DEFAULT_THEME_ID: ThemeId = "mexico-at-night";

export function normalizeThemeId(value: string | null): ThemeId | null {
  if (value === "mexico-sand" || value === "mexico-at-night" || value === "amigo-light-paper") {
    return value;
  }

  if (value === "amigo-mexico") {
    return "mexico-sand";
  }

  if (value === "amigo-dark-navy" || value === "amigo-mexico-dark") {
    return "mexico-at-night";
  }

  return null;
}

export function isThemeId(value: string | null): value is ThemeId {
  return normalizeThemeId(value) === value;
}
