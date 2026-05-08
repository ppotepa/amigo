import { ConsolePanel } from "../../features/scripting/ConsolePanel";
import { RegisteredWindowPanel } from "../../features/system/RegisteredWindowPanel";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type { EditorComponentDefinition, EditorComponentLaunchContext } from "../componentTypes";
import { BOTTOM_DOCK, WINDOW, dockable, windowOnly } from "./shared";

export const ThemeControllerComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  windowOnly({
    id: "theme.controller",
    title: "Theme Controller",
    debugSource: "src/features/system/RegisteredWindowPanel.tsx",
    category: "settings",
    domain: "theme",
    icon: "paintbrush",
    description: "Theme controller window.",
    placement: WINDOW,
    defaultPlacement: WINDOW,
    allowedPlacements: ["window", "modal"],
    render: RegisteredWindowPanel,
  }),
);

export const SettingsGlobalComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  windowOnly({
    id: "settings.global",
    title: "Settings",
    debugSource: "src/features/system/RegisteredWindowPanel.tsx",
    category: "settings",
    domain: "settings",
    icon: "settings",
    description: "Global editor settings.",
    placement: WINDOW,
    defaultPlacement: WINDOW,
    allowedPlacements: ["window"],
    render: RegisteredWindowPanel,
  }),
);

export const CacheManagerComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  windowOnly({
    id: "cache.manager",
    title: "Cache Manager",
    debugSource: "src/features/system/RegisteredWindowPanel.tsx",
    category: "tools",
    domain: "cache",
    icon: "gauge",
    description: "Cache maintenance window.",
    placement: WINDOW,
    defaultPlacement: WINDOW,
    allowedPlacements: ["window"],
    render: RegisteredWindowPanel,
  }),
);

export const ScriptingConsoleComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "scripting.console",
    title: "Console",
    debugSource: "src/features/scripting/ConsolePanel.tsx",
    category: "debug",
    domain: "scripting",
    icon: "terminal",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel", "window"],
    canOpenInWindow: true,
    render: ConsolePanel,
  }),
);

export const SYSTEM_COMPONENTS = [
  ThemeControllerComponent,
  SettingsGlobalComponent,
  CacheManagerComponent,
  ScriptingConsoleComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
