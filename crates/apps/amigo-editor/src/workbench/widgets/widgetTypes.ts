import type React from "react";
import type { EditorTargetKind, EditorTargetRef } from "../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export type WidgetStatus =
  | "ok"
  | "warning"
  | "error"
  | "info"
  | "neutral";

export type WidgetPlacement = "top" | "bottom";

export interface WidgetRenderProps<TModel> {
  model: TModel;
  services: WorkspaceRuntimeServices;
  onSelectTarget?: (target: EditorTargetRef) => void;
}

export interface WidgetDefinition<TModel = unknown> {
  id: string;
  title: string;
  icon?: string;
  domain: string;
  targetKinds: EditorTargetKind[];
  placement: WidgetPlacement;
  order: number;
  getStatus?: (model: TModel) => WidgetStatus;
  getFoldedHint?: (model: TModel) => string;
  render: React.ComponentType<WidgetRenderProps<TModel>>;
}
