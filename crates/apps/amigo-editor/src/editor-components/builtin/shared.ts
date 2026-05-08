import type {
  ComponentPlacement,
  EditorComponentContextPayload,
  EditorComponentDefinition,
} from "../componentTypes";

// @codemap anchor:component-placement-helpers domain:editor-components role:workspace-surface priority:P0 layer:app tags:surface,placement,detached-workspace
export type BuiltinComponentDefinition = EditorComponentDefinition;

export const LEFT_DOCK: ComponentPlacement = { kind: "leftDock" };
export const RIGHT_DOCK: ComponentPlacement = { kind: "rightDock" };
export const BOTTOM_DOCK: ComponentPlacement = { kind: "bottomDock" };
export const CENTER_TAB: ComponentPlacement = { kind: "centerTab" };
export const WINDOW: ComponentPlacement = { kind: "window" };

type BuiltinComponentInput<TContext extends EditorComponentContextPayload = any> = Omit<
  EditorComponentDefinition<TContext>,
  "canDock" | "canFloat" | "canOpenInWindow" | "canOpenInCenterTabs" | "singleton"
> &
  Partial<
    Pick<
      EditorComponentDefinition<TContext>,
      "canDock" | "canFloat" | "canOpenInWindow" | "canOpenInCenterTabs" | "singleton"
    >
  >;

type CenterTabComponentInput<TContext extends EditorComponentContextPayload = any> = Omit<
  BuiltinComponentInput<TContext>,
  "placement" | "defaultPlacement" | "allowedPlacements"
> &
  Partial<Pick<EditorComponentDefinition<TContext>, "placement" | "defaultPlacement" | "allowedPlacements">>;

type WindowComponentInput<TContext extends EditorComponentContextPayload = any> = CenterTabComponentInput<TContext>;

export function dockable<TContext extends EditorComponentContextPayload = any>(
  definition: BuiltinComponentInput<TContext>,
): EditorComponentDefinition<TContext> {
  return {
    canDock: true,
    canFloat: true,
    canOpenInWindow: false,
    canOpenInCenterTabs: false,
    singleton: true,
    ...definition,
  };
}

export function centerTab<TContext extends EditorComponentContextPayload = any>(
  definition: CenterTabComponentInput<TContext>,
): EditorComponentDefinition<TContext> {
  return {
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel"],
    canDock: false,
    canFloat: true,
    canOpenInWindow: false,
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

export function workspaceSurface<TContext extends EditorComponentContextPayload = any>(
  definition: CenterTabComponentInput<TContext>,
): EditorComponentDefinition<TContext> {
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

export function windowOnly<TContext extends EditorComponentContextPayload = any>(
  definition: WindowComponentInput<TContext>,
): EditorComponentDefinition<TContext> {
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
