import { ProjectCapabilitiesPanel } from "../../features/project/ProjectCapabilitiesPanel";
import { ProjectDependenciesPanel } from "../../features/project/ProjectDependenciesPanel";
import { ProjectExplorerPanel } from "../../features/project/ProjectExplorerPanel";
import { ProjectOverviewPanel } from "../../features/project/ProjectOverviewPanel";
import type { EditorComponentDefinition } from "../componentTypes";
import { CENTER_TAB, LEFT_DOCK, dockable, workspaceSurface } from "./shared";

export const PROJECT_COMPONENTS: EditorComponentDefinition[] = [
  dockable({
    id: "project.explorer",
    title: "Project Explorer",
    category: "explorer",
    domain: "project",
    icon: "folder",
    description: "Project file tree and mod structure.",
    placement: LEFT_DOCK,
    defaultPlacement: LEFT_DOCK,
    allowedPlacements: ["leftDock", "rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: ProjectExplorerPanel,
  }),
  workspaceSurface({
    id: "project.overview",
    title: "Mod Overview",
    category: "workspace",
    domain: "project",
    icon: "box",
    description: "Summary of the active mod.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel", "window"],
    requiredContext: ["editorSession"],
    surface: {
      kind: "viewer",
      tabMode: true,
      detachedMode: true,
      detachBehavior: "workspace",
      dockProfileId: "project-overview",
    },
    render: ProjectOverviewPanel,
  }),
  workspaceSurface({
    id: "project.capabilities",
    title: "Capabilities",
    category: "workspace",
    domain: "project",
    icon: "box",
    description: "Capabilities declared by the active mod.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel", "window"],
    requiredContext: ["editorSession"],
    surface: {
      kind: "viewer",
      tabMode: true,
      detachedMode: true,
      detachBehavior: "workspace",
      dockProfileId: "project-overview",
    },
    render: ProjectCapabilitiesPanel,
  }),
  workspaceSurface({
    id: "project.dependencies",
    title: "Dependencies",
    category: "workspace",
    domain: "project",
    icon: "package",
    description: "Dependency state for the active mod.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel", "window"],
    requiredContext: ["editorSession"],
    surface: {
      kind: "viewer",
      tabMode: true,
      detachedMode: true,
      detachBehavior: "workspace",
      dockProfileId: "project-overview",
    },
    render: ProjectDependenciesPanel,
  }),
];
