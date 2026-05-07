import type { ThemeDefinition, ThemeId } from "./themeTypes";

export const THEMES: ThemeDefinition[] = [
  {
    id: "night-in-mexico",
    name: "Night in Mexico",
    description: "Moonlit desert workspace with navy surfaces, agave accents, and amber highlights.",
    mode: "dark",
    accent: "#3A6D73",
  },
  {
    id: "siesta-in-mexico",
    name: "Siesta in Mexico",
    description: "Warm siesta workspace with cream surfaces and soft agave / terracotta accents.",
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
  {
    id: "krakowskie-przedmiescie",
    name: "Krakowskie Przedmieście",
    description: "White stone workspace with Polish flag red accents and quiet city greys.",
    mode: "light",
    accent: "#dc143c",
  },
  {
    id: "miami-sunsets",
    name: "Miami Sunsets",
    description: "Dark Miami dusk workspace with coral borders, magenta glow, cyan accents, and outlined neon controls.",
    mode: "dark",
    accent: "#FF4FA3",
  },
];

export const DEFAULT_THEME_ID: ThemeId = "miami-sunsets";

export function themeNameForId(themeId: ThemeId): string {
  return THEMES.find((theme) => theme.id === themeId)?.name ?? themeId;
}

export function normalizeThemeId(value: string | null): ThemeId | null {
  if (
    value === "night-in-mexico" ||
    value === "siesta-in-mexico" ||
    value === "amigo-light-paper" ||
    value === "krakowskie-przedmiescie" ||
    value === "miami-sunsets"
  ) {
    return value;
  }

  return null;
}

export function isThemeId(value: string | null): value is ThemeId {
  return normalizeThemeId(value) === value;
}
