import type { ComponentType } from "react";
import type { WorkspaceRuntimeServices } from "../main-window/workspaceRuntimeServices";
import type {
  EditorTargetKind,
  ResolvedEditorTarget,
} from "./editorTargetTypes";

export type TargetPanelComponent = ComponentType<{
  target: ResolvedEditorTarget;
  services: WorkspaceRuntimeServices;
}>;

export type TargetPanelInput =
  | TargetPanelComponent
  | readonly TargetPanelComponent[];

export type EditorTargetContextProfileConfig = {
  primary: TargetPanelInput;
  secondary: TargetPanelInput;
  defaultAction?: string;
};

export type EditorTargetContextProfile = {
  primary: TargetPanelComponent[];
  secondary: TargetPanelComponent[];
  defaultAction?: string;
};

export type EditorTargetContextProfileRegistry = Record<
  EditorTargetKind,
  EditorTargetContextProfileConfig
>;

export function normalizeTargetPanelInput(
  input: TargetPanelInput,
): TargetPanelComponent[] {
  return Array.isArray(input)
    ? Array.from(input as readonly TargetPanelComponent[])
    : [input as TargetPanelComponent];
}

export function normalizeEditorTargetContextProfile(
  profile: EditorTargetContextProfileConfig,
): EditorTargetContextProfile {
  return {
    primary: normalizeTargetPanelInput(profile.primary),
    secondary: normalizeTargetPanelInput(profile.secondary),
    defaultAction: profile.defaultAction,
  };
}
