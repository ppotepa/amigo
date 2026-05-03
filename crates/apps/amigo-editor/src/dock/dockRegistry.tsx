import {
  AlertTriangle,
  Box,
  CheckCircle2,
  Folder,
  Gauge,
  Layers3,
  ListTree,
  Package,
  Play,
  Terminal,
} from "lucide-react";
import type { DockPlugin } from "./dockTypes";

export const DOCK_PLUGINS: DockPlugin[] = [
  {
    id: "project-explorer",
    title: "Project",
    icon: <Folder size={14} />,
    defaultDock: "left",
    canOpen: (context) => Boolean(context.modId),
  },
  {
    id: "asset-browser",
    title: "Assets",
    icon: <Package size={14} />,
    defaultDock: "left",
    canOpen: (context) => Boolean(context.modId),
  },
  {
    id: "scene-hierarchy",
    title: "Hierarchy",
    icon: <ListTree size={14} />,
    defaultDock: "left",
    canOpen: (context) => Boolean(context.selectedSceneId),
  },
  {
    id: "scene-preview",
    title: "Scene Preview",
    icon: <Play size={14} />,
    defaultDock: "center",
    canOpen: (context) => Boolean(context.selectedSceneId),
  },
  {
    id: "inspector",
    title: "Inspector",
    icon: <Box size={14} />,
    defaultDock: "right",
    canOpen: (context) => Boolean(context.modId),
  },
  {
    id: "diagnostics",
    title: "Diagnostics",
    icon: <AlertTriangle size={14} />,
    defaultDock: "right",
    canOpen: (context) => Boolean(context.modId),
  },
  {
    id: "properties",
    title: "Properties",
    icon: <Layers3 size={14} />,
    defaultDock: "right",
    canOpen: (context) => Boolean(context.modId),
  },
  {
    id: "problems",
    title: "Problems",
    icon: <AlertTriangle size={14} />,
    defaultDock: "bottom",
    canOpen: () => true,
  },
  {
    id: "event-log",
    title: "Event Log",
    icon: <Terminal size={14} />,
    defaultDock: "bottom",
    canOpen: () => true,
  },
  {
    id: "tasks",
    title: "Tasks",
    icon: <CheckCircle2 size={14} />,
    defaultDock: "bottom",
    canOpen: () => true,
  },
  {
    id: "console",
    title: "Console",
    icon: <Terminal size={14} />,
    defaultDock: "bottom",
    canOpen: () => true,
  },
  {
    id: "preview-cache",
    title: "Preview Cache",
    icon: <Gauge size={14} />,
    defaultDock: "bottom",
    canOpen: (context) => Boolean(context.modId),
  },
];

export function dockPluginById(pluginId: string): DockPlugin | undefined {
  return DOCK_PLUGINS.find((plugin) => plugin.id === pluginId);
}
