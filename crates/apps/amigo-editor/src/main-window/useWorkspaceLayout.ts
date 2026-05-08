import { useEffect, useMemo, useState } from "react";
import { singletonComponentInstanceId } from "../editor-components/componentInstances";
import {
  AssetsBrowserComponent,
  DiagnosticsProblemsComponent,
  DocumentChangesComponent,
  EntityPropertiesComponent,
} from "../editor-components/componentRegistry";
import type { WorkspaceDockLayoutState } from "./workspaceLayout";

const WORKSPACE_LAYOUT_STORAGE_KEY_PREFIX = "amigo-editor.workspace.component-layout.v1";

export type PersistedWorkspaceComponentLayout = {
  leftInstanceId?: string;
  rightInstanceId?: string;
  rightTopInstanceId?: string;
  rightBottomInstanceId?: string;
  bottomInstanceId?: string;
  sizes?: WorkspaceDockSizes;
};

export type WorkspaceDockSizes = {
  leftWidth: number;
  rightWidth: number;
  rightBottomHeight: number;
  bottomHeight: number;
};

export const DEFAULT_WORKSPACE_DOCK_SIZES: WorkspaceDockSizes = {
  leftWidth: 360,
  rightWidth: 380,
  rightBottomHeight: 280,
  bottomHeight: 260,
};

const WORKSPACE_DOCK_SIZE_LIMITS = {
  leftWidth: { min: 240, max: 620 },
  rightWidth: { min: 280, max: 680 },
  rightBottomHeight: { min: 160, max: 520 },
  bottomHeight: { min: 160, max: 520 },
} as const;

const DEFAULT_LEFT_INSTANCE_ID = singletonComponentInstanceId(AssetsBrowserComponent);
const DEFAULT_RIGHT_TOP_INSTANCE_ID = singletonComponentInstanceId(EntityPropertiesComponent);
const DEFAULT_RIGHT_BOTTOM_INSTANCE_ID = singletonComponentInstanceId(DocumentChangesComponent);
const DEFAULT_BOTTOM_INSTANCE_ID = singletonComponentInstanceId(DiagnosticsProblemsComponent);

export function useWorkspaceLayout(workspaceId = "main", initialDockLayout?: WorkspaceDockLayoutState) {
  const storageKey = workspaceLayoutStorageKey(workspaceId);
  const persistedLayout = useMemo(
    () => readPersistedWorkspaceComponentLayout(storageKey),
    [storageKey],
  );
  const [leftInstanceId, setLeftInstanceId] = useState(
    persistedLayout.leftInstanceId ?? initialDockLayout?.leftDock.activeTabId ?? DEFAULT_LEFT_INSTANCE_ID,
  );
  const [rightTopInstanceId, setRightTopInstanceId] = useState(
    persistedLayout.rightTopInstanceId ?? persistedLayout.rightInstanceId ?? initialDockLayout?.rightTopDock.activeTabId ?? DEFAULT_RIGHT_TOP_INSTANCE_ID,
  );
  const [rightBottomInstanceId, setRightBottomInstanceId] = useState(
    persistedLayout.rightBottomInstanceId ?? initialDockLayout?.rightBottomDock.activeTabId ?? DEFAULT_RIGHT_BOTTOM_INSTANCE_ID,
  );
  const [bottomInstanceId, setBottomInstanceId] = useState(
    persistedLayout.bottomInstanceId ?? initialDockLayout?.bottomDock.activeTabId ?? DEFAULT_BOTTOM_INSTANCE_ID,
  );
  const [dockSizes, setDockSizes] = useState<WorkspaceDockSizes>(() =>
    normalizeDockSizes(persistedLayout.sizes, initialDockLayout),
  );

  useEffect(() => {
    persistWorkspaceComponentLayout(storageKey, {
      bottomInstanceId,
      leftInstanceId,
      rightBottomInstanceId,
      rightTopInstanceId,
      sizes: dockSizes,
    });
  }, [bottomInstanceId, dockSizes, leftInstanceId, rightBottomInstanceId, rightTopInstanceId, storageKey]);

  function resizeDock(sizeKey: keyof WorkspaceDockSizes, delta: number) {
    setDockSizes((current) => ({
      ...current,
      [sizeKey]: clampDockSize(sizeKey, current[sizeKey] + delta),
    }));
  }

  function resetDockSize(sizeKey: keyof WorkspaceDockSizes) {
    setDockSizes((current) => ({
      ...current,
      [sizeKey]: DEFAULT_WORKSPACE_DOCK_SIZES[sizeKey],
    }));
  }

  function resetLayout() {
    setDockSizes(DEFAULT_WORKSPACE_DOCK_SIZES);
  }

  return {
    bottomInstanceId,
    dockSizes,
    leftInstanceId,
    resetDockSize,
    resetLayout,
    resizeDock,
    rightBottomInstanceId,
    rightTopInstanceId,
    setBottomInstanceId,
    setLeftInstanceId,
    setRightBottomInstanceId,
    setRightTopInstanceId,
  };
}

function workspaceLayoutStorageKey(workspaceId: string): string {
  return `${WORKSPACE_LAYOUT_STORAGE_KEY_PREFIX}.${workspaceId || "main"}`;
}

function readPersistedWorkspaceComponentLayout(storageKey: string): PersistedWorkspaceComponentLayout {
  try {
    const text = window.localStorage.getItem(storageKey);
    return text ? (JSON.parse(text) as PersistedWorkspaceComponentLayout) : {};
  } catch {
    return {};
  }
}

function persistWorkspaceComponentLayout(storageKey: string, layout: PersistedWorkspaceComponentLayout) {
  window.localStorage.setItem(storageKey, JSON.stringify(layout));
}

function normalizeDockSizes(sizes?: Partial<WorkspaceDockSizes>, initialDockLayout?: WorkspaceDockLayoutState): WorkspaceDockSizes {
  return {
    leftWidth: clampDockSize("leftWidth", sizes?.leftWidth ?? initialDockLayout?.leftDock.size ?? DEFAULT_WORKSPACE_DOCK_SIZES.leftWidth),
    rightWidth: clampDockSize("rightWidth", sizes?.rightWidth ?? initialDockLayout?.rightTopDock.size ?? DEFAULT_WORKSPACE_DOCK_SIZES.rightWidth),
    rightBottomHeight: clampDockSize("rightBottomHeight", sizes?.rightBottomHeight ?? initialDockLayout?.rightBottomDock.size ?? DEFAULT_WORKSPACE_DOCK_SIZES.rightBottomHeight),
    bottomHeight: clampDockSize("bottomHeight", sizes?.bottomHeight ?? initialDockLayout?.bottomDock.size ?? DEFAULT_WORKSPACE_DOCK_SIZES.bottomHeight),
  };
}

function clampDockSize(sizeKey: keyof WorkspaceDockSizes, value: number): number {
  const limits = WORKSPACE_DOCK_SIZE_LIMITS[sizeKey];
  return Math.min(limits.max, Math.max(limits.min, Math.round(value)));
}
