export type ThemeId = "amigo-dark-navy" | "amigo-light-paper";

export interface ThemeDefinition {
  id: ThemeId;
  name: string;
  description: string;
  mode: "dark" | "light";
  accent: string;
}
