import type { WorkspaceProjectNodeRef } from "../../main-window/workspaceRuntimeServices";

export type ProjectNodeActionContext = {
  openCenterComponent: (componentId: string) => void;
  showBottomPanel: (instanceId: string) => void;
  validateSelectedMod: () => Promise<void>;
};

export type ProjectNodeAction = {
  id: string;
  canRun: (node: WorkspaceProjectNodeRef) => boolean;
  run: (node: WorkspaceProjectNodeRef, context: ProjectNodeActionContext) => void | Promise<void>;
};

export const PROJECT_NODE_ACTIONS: readonly ProjectNodeAction[] = [
  {
    id: "project.openOverview",
    canRun: (node) => node.kind === "overview",
    run: (_node, context) => context.openCenterComponent("project.overview"),
  },
  {
    id: "project.openCapabilities",
    canRun: (node) => node.kind === "capabilities",
    run: (_node, context) => context.openCenterComponent("project.capabilities"),
  },
  {
    id: "project.openDependencies",
    canRun: (node) => node.kind === "dependencies",
    run: (_node, context) => context.openCenterComponent("project.dependencies"),
  },
  {
    id: "project.showDiagnostics",
    canRun: (node) => node.kind === "diagnostics",
    run: (_node, context) => context.showBottomPanel("diagnostics.problems:singleton"),
  },
  {
    id: "project.validateManifest",
    canRun: (node) => node.kind === "manifest",
    run: async (_node, context) => context.validateSelectedMod(),
  },
] as const;
