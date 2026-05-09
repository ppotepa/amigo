import type { ReactNode } from "react";

export type TabSpec = {
  id: string;
  title: string;
  icon?: ReactNode;
  content: ReactNode;
};

export type SplitSlotContent = {
  title?: string;
  subtitle?: string;
  tabs: TabSpec[];
  activeTabId?: string;
  onTabChange?: (tabId: string) => void;
};

export function isSplitSlotContent(value: unknown): value is SplitSlotContent {
  return Boolean(
    value &&
    typeof value === "object" &&
    "tabs" in value &&
    Array.isArray((value as SplitSlotContent).tabs),
  );
}
