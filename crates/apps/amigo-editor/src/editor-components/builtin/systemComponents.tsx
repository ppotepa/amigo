import { ConsolePanel } from "../../features/scripting/ConsolePanel";
import { RegisteredWindowPanel } from "../../features/system/RegisteredWindowPanel";
import type { EditorComponentDefinition } from "../componentTypes";
import { BOTTOM_DOCK, WINDOW, dockable, windowOnly } from "./shared";

export const SYSTEM_COMPONENTS: EditorComponentDefinition[] = [
  windowOnly({
    id: "theme.controller",
    title: "Theme Controller",
    category: "settings",
    domain: "theme",
    icon: "paintbrush",
    description: "Theme controller window.",
    placement: WINDOW,
    defaultPlacement: WINDOW,
    allowedPlacements: ["window", "modal"],
    render: RegisteredWindowPanel,
  }),
  windowOnly({
    id: "settings.global",
    title: "Settings",
    category: "settings",
    domain: "settings",
    icon: "settings",
    description: "Global editor settings.",
    placement: WINDOW,
    defaultPlacement: WINDOW,
    allowedPlacements: ["window"],
    render: RegisteredWindowPanel,
  }),
  windowOnly({
    id: "cache.manager",
    title: "Cache Manager",
    category: "tools",
    domain: "cache",
    icon: "gauge",
    description: "Cache maintenance window.",
    placement: WINDOW,
    defaultPlacement: WINDOW,
    allowedPlacements: ["window"],
    render: RegisteredWindowPanel,
  }),
  dockable({
    id: "scripting.console",
    title: "Console",
    category: "debug",
    domain: "scripting",
    icon: "terminal",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel", "window"],
    canOpenInWindow: true,
    render: ConsolePanel,
  }),
];
