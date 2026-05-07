import type React from "react";

// @codemap anchor:component-surface-types domain:editor-components role:workspace-surface priority:P0 layer:app tags:surface,workspace,detached-workspace
export type EditorComponentCategory =
  | "workspace"
  | "explorer"
  | "inspector"
  | "ui"
  | "editor"
  | "preview"
  | "diagnostics"
  | "tools"
  | "settings"
  | "system"
  | "debug";

export type EditorComponentDomain =
  | "editor"
  | "ui"
  | "project"
  | "modding"
  | "scene"
  | "assets"
  | "scripting"
  | "preview"
  | "cache"
  | "theme"
  | "settings"
  | "windowing"
  | "diagnostics"
  | "runtime"
  | "rendering_2d"
  | "motion_2d"
  | "physics_2d"
  | "particles_2d"
  | "audio"
  | "tilemap"
  | "tileset";

export type IconKey =
  | "alert-triangle"
  | "box"
  | "check-circle"
  | "folder"
  | "gauge"
  | "grid"
  | "layers"
  | "list"
  | "list-tree"
  | "layout-template"
  | "package"
  | "paintbrush"
  | "plus"
  | "play"
  | "refresh"
  | "image"
  | "settings"
  | "terminal"
  | "type";

export type ComponentPlacementKind =
  | "leftDock"
  | "rightDock"
  | "bottomDock"
  | "centerTab"
  | "floatingPanel"
  | "window"
  | "modal";

export interface ComponentPlacement {
  kind: ComponentPlacementKind;
  area?: string;
}

export type ComponentContextRequirement =
  | "editorSession"
  | "selectedMod"
  | "selectedScene"
  | "selectedAsset"
  | "selectedEntity"
  | "projectCache"
  | "runtimePreview";

export interface EditorComponentContext {
  sessionId: string | null;
  modId: string | null;
  selectedSceneId: string | null;
  selectedEntityId?: string | null;
  selectedAssetId?: string | null;
  capabilities?: string[];
}

export interface EditorComponentProps<TServices = any> {
  instance: EditorComponentInstance;
  context: EditorComponentContext;
  services: TServices;
}

export type ComponentToolbarValue = string | boolean;

export type ComponentToolbarState = Record<string, ComponentToolbarValue>;

export type ComponentToolbarControl =
  | {
      kind: "spacer";
      id: string;
    }
  | {
      kind: "segmented";
      id: string;
      label: string;
      defaultValue: string;
      options: Array<{ id: string; label: string; icon: IconKey }>;
    }
  | {
      kind: "toggle";
      id: string;
      label: string;
      icon: IconKey;
      defaultValue: boolean;
    }
  | {
      kind: "select";
      id: string;
      label: string;
      defaultValue: string;
      options: Array<{ id: string; label: string }>;
    }
  | {
      kind: "action";
      id: string;
      label: string;
      icon: IconKey;
    };

export interface EditorComponentToolbarDefinition {
  compact?: boolean;
  controls: ComponentToolbarControl[];
}

export type EditorSurfaceKind = "editor" | "viewer" | "tool" | "settings";

export type DetachBehavior = "workspace" | "componentWindow" | "disabled";

export interface EditorSurfaceDefinition {
  kind: EditorSurfaceKind;
  tabMode: boolean;
  detachedMode: boolean;
  detachBehavior: DetachBehavior;
  dockProfileId?: string;
}

export interface EditorComponentDefinition {
  id: string;
  title: string;
  debugSource?: string;
  category: EditorComponentCategory;
  domain: EditorComponentDomain;
  subdomain?: string;
  icon: IconKey;
  description?: string;
  placement: ComponentPlacement;
  defaultPlacement: ComponentPlacement;
  allowedPlacements: ComponentPlacementKind[];
  capabilities?: string[];
  requiredContext?: ComponentContextRequirement[];
  canDock: boolean;
  canFloat: boolean;
  canOpenInWindow: boolean;
  canOpenInCenterTabs: boolean;
  singleton: boolean;
  surface?: EditorSurfaceDefinition;
  toolbar?: EditorComponentToolbarDefinition;
  defaultSize?: {
    width?: number;
    height?: number;
    minWidth?: number;
    minHeight?: number;
  };
  render: React.ComponentType<EditorComponentProps>;
}

export interface EditorComponentInstance {
  instanceId: string;
  componentId: string;
  sessionId?: string;
  resourceUri?: string;
  context?: Record<string, string>;
  titleOverride?: string;
  dirty?: boolean;
  placement: ComponentPlacement;
}
