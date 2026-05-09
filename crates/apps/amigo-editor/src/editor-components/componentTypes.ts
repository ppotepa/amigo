import type React from "react";
import type { ResolvedEditorTarget } from "../editor-targets";
import type { WorkspaceRuntimeServices } from "../main-window/workspaceRuntimeServices";

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

export interface EditorComponentProps<
  TServices = any,
  TContext extends EditorComponentContextPayload = any,
> {
  instance: EditorComponentInstance<TContext>;
  context: TContext;
  services: TServices;
}

export type EditorComponentContextValue = string | number | boolean | null | undefined;
export type EditorComponentContextPayload = Record<string, EditorComponentContextValue>;
export type EditorSerializedComponentContext = Record<string, string>;

export type EditorComponentLaunchContext = {
  modId?: string;
  sessionId?: string;
  sceneId?: string;
};

export type ScenePreviewComponentContext = EditorComponentLaunchContext & {
  sceneId?: string;
};

export type FileWorkspaceComponentContext = EditorComponentLaunchContext & {
  fileKind: string;
  filePath: string;
};

export type UiDocumentEditorContext = EditorComponentLaunchContext & {
  sceneId: string;
  entityId?: string;
  componentIndex?: number;
  focusPath?: string;
  preferredEntityId?: string;
  initialTemplate: string;
};
export type EditorComponentId = string;

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

export type EditorComponentPresentation = {
  title?: string;
  icon?: IconKey;
};

export interface EditorRenderableDefinition<
  TProps = any,
  TContext extends EditorComponentContextPayload = any,
> {
  id: string;
  title: string;
  icon: IconKey;
  domain: EditorComponentDomain;
  description?: string;
  render: React.ComponentType<TProps>;
  readonly __context?: TContext;
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

export interface EditorComponentDefinition<TContext extends EditorComponentContextPayload = any>
  extends EditorRenderableDefinition<EditorComponentProps<any, TContext>, TContext> {
  debugSource?: string;
  category: EditorComponentCategory;
  subdomain?: string;
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
  canRenderTarget?: (
    target: ResolvedEditorTarget,
    services: WorkspaceRuntimeServices,
  ) => boolean;
  contextFromTarget?: (
    target: ResolvedEditorTarget,
    services: WorkspaceRuntimeServices,
  ) => TContext | null;
  presentationFromTarget?: (
    target: ResolvedEditorTarget,
    services: WorkspaceRuntimeServices,
  ) => EditorComponentPresentation;
  defaultSize?: {
    width?: number;
    height?: number;
    minWidth?: number;
    minHeight?: number;
  };
}

export type AnyEditorComponentDefinition = EditorComponentDefinition<any>;

export type EditorComponentContextOf<TComponent> =
  TComponent extends EditorComponentDefinition<infer TContext> ? TContext : never;

export type EditorComponentOpenRequest<
  TComponent extends EditorComponentDefinition<any> = EditorComponentDefinition<any>,
> = {
  component: TComponent;
  context?: EditorComponentContextOf<TComponent>;
};

export interface EditorComponentInstance<TContext extends EditorComponentContextPayload = any> {
  instanceId: string;
  component: EditorComponentDefinition<TContext>;
  /** Serialized stable id for layout, URL and detached-window boundaries. Prefer `component` in app code. */
  componentId: string;
  sessionId?: string;
  resourceUri?: string;
  context?: TContext;
  titleOverride?: string;
  iconOverride?: IconKey;
  dirty?: boolean;
  placement: ComponentPlacement;
}
