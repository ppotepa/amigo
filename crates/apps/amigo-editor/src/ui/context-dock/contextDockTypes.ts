import type { ReactNode } from "react";

export type ContextWidgetId = string;

export type ContextWidgetState = {
  id: ContextWidgetId;
  collapsed: boolean;
  search?: string;
};

export type ContextWidgetStateMap = Record<ContextWidgetId, ContextWidgetState>;

export type ContextBadgeTone =
  | "muted"
  | "info"
  | "valid"
  | "warning"
  | "error";

export type ContextRowTone =
  | "default"
  | "blue"
  | "cyan"
  | "green"
  | "orange"
  | "purple"
  | "red";

export type ContextAction = {
  id: string;
  label: string;
  icon?: ReactNode;
  title?: string;
  disabled?: boolean;
  onClick: () => void;
};

export type ContextTreeNode = {
  id: string;
  title: string;
  subtitle?: string;
  icon?: ReactNode;
  badge?: ReactNode;
  selected?: boolean;
  defaultExpanded?: boolean;
  actions?: ReactNode;
  children?: ContextTreeNode[];
  onSelect?: () => void;
};
