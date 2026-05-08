import { CachePanel } from "../../features/cache/CachePanel";
import { DiagnosticsPanel } from "../../features/diagnostics/DiagnosticsPanel";
import { ProblemsTable } from "../../features/diagnostics/ProblemsTable";
import { EventTable } from "../../features/events/EventTable";
import { TaskTable } from "../../features/tasks/TaskTable";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type { EditorComponentDefinition, EditorComponentLaunchContext } from "../componentTypes";
import { BOTTOM_DOCK, RIGHT_DOCK, dockable } from "./shared";

export const DiagnosticsProblemsComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "diagnostics.problems",
    title: "Problems",
    debugSource: "src/features/diagnostics/ProblemsTable.tsx",
    category: "diagnostics",
    domain: "diagnostics",
    icon: "alert-triangle",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel", "window"],
    canOpenInWindow: true,
    toolbar: {
      compact: true,
      controls: [
        {
          kind: "segmented",
          id: "level",
          label: "Level",
          defaultValue: "all",
          options: [
            { id: "all", label: "All", icon: "list" },
            { id: "error", label: "Errors", icon: "alert-triangle" },
            { id: "warning", label: "Warnings", icon: "gauge" },
          ],
        },
      ],
    },
    render: ProblemsTable,
  }),
);

export const DiagnosticsPanelComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "diagnostics.panel",
    title: "Diagnostics",
    debugSource: "src/features/diagnostics/DiagnosticsPanel.tsx",
    category: "diagnostics",
    domain: "diagnostics",
    icon: "alert-triangle",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "bottomDock", "floatingPanel", "window"],
    canOpenInWindow: true,
    render: DiagnosticsPanel,
  }),
);

export const EventsLogComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "events.log",
    title: "Event Log",
    debugSource: "src/features/events/EventTable.tsx",
    category: "system",
    domain: "windowing",
    subdomain: "event-bus",
    icon: "terminal",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel", "window"],
    canOpenInWindow: true,
    toolbar: {
      compact: true,
      controls: [
        {
          kind: "select",
          id: "category",
          label: "Category",
          defaultValue: "all",
          options: [
            { id: "all", label: "All" },
            { id: "window", label: "Window" },
            { id: "asset", label: "Asset" },
            { id: "workspace", label: "Workspace" },
            { id: "cache", label: "Cache" },
            { id: "settings", label: "Settings" },
          ],
        },
      ],
    },
    render: EventTable,
  }),
);

export const TasksMonitorComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "tasks.monitor",
    title: "Tasks",
    debugSource: "src/features/tasks/TaskTable.tsx",
    category: "system",
    domain: "editor",
    subdomain: "tasks",
    icon: "check-circle",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel", "window"],
    canOpenInWindow: true,
    render: TaskTable,
  }),
);

export const CachePreviewComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "cache.preview",
    title: "Preview Cache",
    debugSource: "src/features/cache/CachePanel.tsx",
    category: "system",
    domain: "cache",
    icon: "gauge",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel", "window"],
    requiredContext: ["projectCache"],
    canOpenInWindow: true,
    render: CachePanel,
  }),
);

export const DIAGNOSTICS_COMPONENTS = [
  DiagnosticsProblemsComponent,
  DiagnosticsPanelComponent,
  EventsLogComponent,
  TasksMonitorComponent,
  CachePreviewComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
