import type React from "react";
import type { EditorTargetKind, EditorTargetRef } from "../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export type WidgetStatus =
  | "ok"
  | "warning"
  | "error"
  | "info"
  | "neutral"
  | "valid"
  | "muted";

export type WidgetBadgeTone =
  | "ok"
  | "warning"
  | "error"
  | "info"
  | "neutral"
  | "muted";

export type HeaderWidgetBadgeTone = WidgetBadgeTone;

export type HeaderWidgetBadge = {
  id: string;
  label: string;
  tone: HeaderWidgetBadgeTone;
};

export type HeaderWidgetModel = {
  title: string;
  subtitle?: string;
  status?: WidgetStatus | WidgetBadgeTone;
  foldedHint?: string;
  badges?: HeaderWidgetBadge[];
  editableTitle?: boolean;
};

export type AssetUsageWidgetItem = {
  label: string;
  count: number;
};

export type AssetUsageWidgetModel = {
  items: AssetUsageWidgetItem[];
};

export type FileInfoWidgetModel = {
  path?: string | null;
  scriptPath?: string | null;
  dirty: boolean;
  type?: string | null;
  modifiedAt?: string | null;
};

export type PropertiesWidgetModel = {
  fallbackText?: string;
  targetLabel?: string;
  target?: unknown;
};

export type DiagnosticsWidgetModel = {
  diagnostics?: Array<{
    id?: string;
    level?: string;
    message?: string;
  }> | null;
};

export type ChangesWidgetModel = {
  dirty?: boolean;
  summary?: string;
};

export type ComponentsWidgetItem = {
  id: string;
  label: string;
  subtitle?: string;
  status: WidgetStatus;
  target?: unknown;
};

export type ComponentsWidgetGroup = {
  id: string;
  label: string;
  count: number;
  items: ComponentsWidgetItem[];
  status?: WidgetStatus;
};

export type ComponentsWidgetModel = {
  title: string;
  foldedHint: string;
  total: number;
  warningCount: number;
  groups: ComponentsWidgetGroup[];
};

export type ChildrenWidgetItem = ComponentsWidgetItem;

export type ChildrenWidgetGroup = {
  id: string;
  label: string;
  count: number;
  items: ChildrenWidgetItem[];
  status?: WidgetStatus;
};

export type ChildrenWidgetModel = {
  title: string;
  foldedHint: string;
  total: number;
  warningCount: number;
  groups: ChildrenWidgetGroup[];
};

export type FlowLinkRow = {
  id: string;
  label: string;
  subtitle?: string;
  status: WidgetStatus;
  target?: unknown;
};

export type FlowLinksWidgetModel = {
  title: string;
  foldedHint: string;
  incoming: FlowLinkRow[];
  outgoing: FlowLinkRow[];
  entries: FlowLinkRow[];
  triggers: FlowLinkRow[];
};

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
