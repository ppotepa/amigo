export type ThemeId = "night-in-mexico" | "siesta-in-mexico" | "amigo-light-paper" | "krakowskie-przedmiescie";

export interface ThemeDefinition {
  id: ThemeId;
  name: string;
  description: string;
  mode: "dark" | "light";
  accent: string;
}
