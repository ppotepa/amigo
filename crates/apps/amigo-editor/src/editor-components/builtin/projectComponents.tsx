import { ProjectCapabilitiesPanel } from "../../features/project/ProjectCapabilitiesPanel";
import { ProjectDependenciesPanel } from "../../features/project/ProjectDependenciesPanel";
import { ProjectExplorerPanel } from "../../features/project/ProjectExplorerPanel";
import { ProjectOverviewPanel } from "../../features/project/ProjectOverviewPanel";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type { EditorComponentDefinition, EditorComponentLaunchContext } from "../componentTypes";
import { CENTER_TAB, LEFT_DOCK, dockable, workspaceSurface } from "./shared";

export const ProjectExplorerComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "project.explorer",
    title: "Project Explorer",
    debugSource: "src/features/project/ProjectExplorerPanel.tsx",
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
);

export const ProjectOverviewComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  workspaceSurface({
    id: "project.overview",
    title: "Mod Overview",
    debugSource: "src/features/project/ProjectOverviewPanel.tsx",
    category: "workspace",
    domain: "project",
    icon: "box",
    description: "Summary of the active mod.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel"],
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
);

export const ProjectCapabilitiesComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  workspaceSurface({
    id: "project.capabilities",
    title: "Capabilities",
    debugSource: "src/features/project/ProjectCapabilitiesPanel.tsx",
    category: "workspace",
    domain: "project",
    icon: "box",
    description: "Capabilities declared by the active mod.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel"],
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
);

export const ProjectDependenciesComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  workspaceSurface({
    id: "project.dependencies",
    title: "Dependencies",
    debugSource: "src/features/project/ProjectDependenciesPanel.tsx",
    category: "workspace",
    domain: "project",
    icon: "package",
    description: "Dependency state for the active mod.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel"],
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
);

export const PROJECT_COMPONENTS = [
  ProjectExplorerComponent,
  ProjectOverviewComponent,
  ProjectCapabilitiesComponent,
  ProjectDependenciesComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
