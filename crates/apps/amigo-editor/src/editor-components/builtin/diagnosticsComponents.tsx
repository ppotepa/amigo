import { CachePanel } from "../../features/cache/CachePanel";
import { DiagnosticsPanel } from "../../features/diagnostics/DiagnosticsPanel";
import { ProblemsTable } from "../../features/diagnostics/ProblemsTable";
import { EventTable } from "../../features/events/EventTable";
import { TaskTable } from "../../features/tasks/TaskTable";
import type { EditorComponentDefinition } from "../componentTypes";
import { BOTTOM_DOCK, RIGHT_DOCK, dockable } from "./shared";

export const DIAGNOSTICS_COMPONENTS: EditorComponentDefinition[] = [
  dockable({
    id: "diagnostics.problems",
    title: "Problems",
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
  dockable({
    id: "diagnostics.panel",
    title: "Diagnostics",
    category: "diagnostics",
    domain: "diagnostics",
    icon: "alert-triangle",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "bottomDock", "floatingPanel", "window"],
    canOpenInWindow: true,
    render: DiagnosticsPanel,
  }),
  dockable({
    id: "events.log",
    title: "Event Log",
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
  dockable({
    id: "tasks.monitor",
    title: "Tasks",
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
  dockable({
    id: "cache.preview",
    title: "Preview Cache",
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
];
