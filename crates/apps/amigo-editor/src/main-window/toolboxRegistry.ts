import {
  AlertTriangle,
  Command,
  Crosshair,
  Eye,
  Maximize2,
  PanelsTopLeft,
  Play,
  RefreshCcw,
  Save,
  ShieldCheck,
  SlidersHorizontal,
  Terminal,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import type { EditorModDetailsDto, EditorProjectFileDto, EditorSceneSummaryDto } from "../api/dto";

export type WorkspaceToolboxActionId =
  | "preview.toggle"
  | "preview.fit"
  | "file.reveal"
  | "file.save"
  | "command.palette"
  | "layout.reset"
  | "panel.problems"
  | "panel.events"
  | "window.fullscreen"
  | "toolbox.configure"
  | "preview.regenerate"
  | "mod.validate";

export type WorkspaceToolboxContext = {
  details: EditorModDetailsDto | null;
  selectedScene: EditorSceneSummaryDto | null;
  selectedFile: EditorProjectFileDto | null;
};

export type WorkspaceToolboxAction = {
  id: WorkspaceToolboxActionId;
  label: string;
  icon: LucideIcon;
  group: "preview" | "file" | "workspace" | "panels" | "system";
  enabled?: (context: WorkspaceToolboxContext) => boolean;
};

export const WORKSPACE_TOOLBOX_ACTIONS: WorkspaceToolboxAction[] = [
  {
    id: "preview.toggle",
    label: "Play / pause preview",
    icon: Play,
    group: "preview",
    enabled: ({ details, selectedScene }) => Boolean(details && selectedScene),
  },
  {
    id: "preview.fit",
    label: "Fit preview to workspace",
    icon: Crosshair,
    group: "preview",
    enabled: ({ selectedScene }) => Boolean(selectedScene),
  },
  {
    id: "file.reveal",
    label: "Reveal selected file",
    icon: Eye,
    group: "file",
    enabled: ({ selectedFile }) => Boolean(selectedFile),
  },
  {
    id: "file.save",
    label: "Save current file",
    icon: Save,
    group: "file",
    enabled: ({ selectedFile }) => Boolean(selectedFile),
  },
  {
    id: "command.palette",
    label: "Open command palette",
    icon: Command,
    group: "workspace",
  },
  {
    id: "layout.reset",
    label: "Reset layout",
    icon: PanelsTopLeft,
    group: "workspace",
  },
  {
    id: "panel.problems",
    label: "Show problems",
    icon: AlertTriangle,
    group: "panels",
  },
  {
    id: "panel.events",
    label: "Show event log",
    icon: Terminal,
    group: "panels",
  },
  {
    id: "window.fullscreen",
    label: "Toggle fullscreen",
    icon: Maximize2,
    group: "system",
  },
  {
    id: "toolbox.configure",
    label: "Configure toolbox",
    icon: SlidersHorizontal,
    group: "system",
  },
  {
    id: "preview.regenerate",
    label: "Regenerate scene preview",
    icon: RefreshCcw,
    group: "preview",
    enabled: ({ details, selectedScene }) => Boolean(details && selectedScene),
  },
  {
    id: "mod.validate",
    label: "Validate mod",
    icon: ShieldCheck,
    group: "workspace",
    enabled: ({ details }) => Boolean(details),
  },
];

export const DEFAULT_WORKSPACE_TOOLBOX_ACTION_IDS: WorkspaceToolboxActionId[] = [
  "preview.toggle",
  "preview.fit",
  "file.reveal",
  "command.palette",
  "panel.problems",
  "panel.events",
  "window.fullscreen",
  "toolbox.configure",
];
