import type { ReactNode } from "react";

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
