import { useEffect, useMemo, useState } from "react";

const WORKSPACE_LAYOUT_STORAGE_KEY = "amigo-editor.workspace.component-layout.v1";

export type PersistedWorkspaceComponentLayout = {
  leftInstanceId?: string;
  rightInstanceId?: string;
  bottomInstanceId?: string;
  sizes?: WorkspaceDockSizes;
};

export type WorkspaceDockSizes = {
  leftWidth: number;
  rightWidth: number;
  bottomHeight: number;
};

export const DEFAULT_WORKSPACE_DOCK_SIZES: WorkspaceDockSizes = {
  leftWidth: 360,
  rightWidth: 380,
  bottomHeight: 260,
};

const WORKSPACE_DOCK_SIZE_LIMITS = {
  leftWidth: { min: 240, max: 620 },
  rightWidth: { min: 280, max: 680 },
  bottomHeight: { min: 160, max: 520 },
} as const;

export function useWorkspaceLayout() {
  const persistedLayout = useMemo(readPersistedWorkspaceComponentLayout, []);
  const [leftInstanceId, setLeftInstanceId] = useState(
    persistedLayout.leftInstanceId ?? "assets.browser:singleton",
  );
  const [rightInstanceId, setRightInstanceId] = useState(
    persistedLayout.rightInstanceId ?? "entity.inspector:singleton",
  );
  const [bottomInstanceId, setBottomInstanceId] = useState(
    persistedLayout.bottomInstanceId ?? "diagnostics.problems:singleton",
  );
  const [dockSizes, setDockSizes] = useState<WorkspaceDockSizes>(() =>
    normalizeDockSizes(persistedLayout.sizes),
  );

  useEffect(() => {
    persistWorkspaceComponentLayout({
      bottomInstanceId,
      leftInstanceId,
      rightInstanceId,
      sizes: dockSizes,
    });
  }, [bottomInstanceId, dockSizes, leftInstanceId, rightInstanceId]);

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
    rightInstanceId,
    setBottomInstanceId,
    setLeftInstanceId,
    setRightInstanceId,
  };
}

function readPersistedWorkspaceComponentLayout(): PersistedWorkspaceComponentLayout {
  try {
    const text = window.localStorage.getItem(WORKSPACE_LAYOUT_STORAGE_KEY);
    return text ? (JSON.parse(text) as PersistedWorkspaceComponentLayout) : {};
  } catch {
    return {};
  }
}

function persistWorkspaceComponentLayout(layout: PersistedWorkspaceComponentLayout) {
  window.localStorage.setItem(WORKSPACE_LAYOUT_STORAGE_KEY, JSON.stringify(layout));
}

function normalizeDockSizes(sizes?: Partial<WorkspaceDockSizes>): WorkspaceDockSizes {
  return {
    leftWidth: clampDockSize("leftWidth", sizes?.leftWidth ?? DEFAULT_WORKSPACE_DOCK_SIZES.leftWidth),
    rightWidth: clampDockSize("rightWidth", sizes?.rightWidth ?? DEFAULT_WORKSPACE_DOCK_SIZES.rightWidth),
    bottomHeight: clampDockSize("bottomHeight", sizes?.bottomHeight ?? DEFAULT_WORKSPACE_DOCK_SIZES.bottomHeight),
  };
}

function clampDockSize(sizeKey: keyof WorkspaceDockSizes, value: number): number {
  const limits = WORKSPACE_DOCK_SIZE_LIMITS[sizeKey];
  return Math.min(limits.max, Math.max(limits.min, Math.round(value)));
}
