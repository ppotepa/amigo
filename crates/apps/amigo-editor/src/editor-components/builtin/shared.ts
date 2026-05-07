import type { ComponentPlacement, EditorComponentDefinition } from "../componentTypes";

export type BuiltinComponentDefinition = EditorComponentDefinition;

export const LEFT_DOCK: ComponentPlacement = { kind: "leftDock" };
export const RIGHT_DOCK: ComponentPlacement = { kind: "rightDock" };
export const BOTTOM_DOCK: ComponentPlacement = { kind: "bottomDock" };
export const CENTER_TAB: ComponentPlacement = { kind: "centerTab" };
export const WINDOW: ComponentPlacement = { kind: "window" };

type BuiltinComponentInput = Omit<
  EditorComponentDefinition,
  "canDock" | "canFloat" | "canOpenInWindow" | "canOpenInCenterTabs" | "singleton"
> &
  Partial<
    Pick<
      EditorComponentDefinition,
      "canDock" | "canFloat" | "canOpenInWindow" | "canOpenInCenterTabs" | "singleton"
    >
  >;

type CenterTabComponentInput = Omit<
  BuiltinComponentInput,
  "placement" | "defaultPlacement" | "allowedPlacements"
> &
  Partial<Pick<EditorComponentDefinition, "placement" | "defaultPlacement" | "allowedPlacements">>;

type WindowComponentInput = CenterTabComponentInput;

export function dockable(definition: BuiltinComponentInput): EditorComponentDefinition {
  return {
    canDock: true,
    canFloat: true,
    canOpenInWindow: false,
    canOpenInCenterTabs: false,
    singleton: true,
    ...definition,
  };
}

export function centerTab(definition: CenterTabComponentInput): EditorComponentDefinition {
  return {
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel", "window"],
    canDock: false,
    canFloat: true,
    canOpenInWindow: true,
    canOpenInCenterTabs: true,
    singleton: true,
    surface: {
      kind: "editor",
      tabMode: true,
      detachedMode: true,
      detachBehavior: "workspace",
      dockProfileId: "default-editor",
    },
    ...definition,
  };
}

export function workspaceSurface(definition: CenterTabComponentInput): EditorComponentDefinition {
  const surface = definition.surface ?? {
    kind: "editor",
    tabMode: true,
    detachedMode: true,
    detachBehavior: "workspace",
    dockProfileId: "default-editor",
  };

  return centerTab({
    ...definition,
    surface: {
      ...surface,
      tabMode: true,
      detachedMode: true,
      detachBehavior: "workspace",
      dockProfileId: surface.dockProfileId ?? "default-editor",
    },
  });
}

export function windowOnly(definition: WindowComponentInput): EditorComponentDefinition {
  return {
    placement: WINDOW,
    defaultPlacement: WINDOW,
    allowedPlacements: ["window"],
    canDock: false,
    canFloat: false,
    canOpenInWindow: true,
    canOpenInCenterTabs: false,
    singleton: true,
    surface: {
      kind: "settings",
      tabMode: false,
      detachedMode: false,
      detachBehavior: "componentWindow",
    },
    ...definition,
  };
}
