export type ThemeId = "amigo-dark-navy" | "amigo-light-paper" | "amigo-mexico" | "amigo-mexico-dark";

export interface ThemeDefinition {
  id: ThemeId;
  name: string;
  description: string;
  mode: "dark" | "light";
  accent: string;
}
