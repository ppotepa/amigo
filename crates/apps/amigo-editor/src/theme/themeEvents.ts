import type { ThemeId } from "./themeTypes";

export type ThemeEvent =
  | { type: "ThemeControllerOpened" }
  | { type: "ThemePreviewStarted"; themeId: ThemeId }
  | { type: "ThemePreviewCancelled" }
  | { type: "ThemeApplyRequested"; themeId: ThemeId }
  | { type: "ThemeApplied"; themeId: ThemeId }
  | { type: "ThemeApplyFailed"; themeId: ThemeId; error: string };
